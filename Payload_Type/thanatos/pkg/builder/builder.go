package builder

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/MythicAgents/thanatos/pkg/builder/types"
	thanatoserror "github.com/MythicAgents/thanatos/pkg/errors"
	"github.com/MythicAgents/thanatos/proto/config"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
	"github.com/google/uuid"
	"golang.org/x/exp/slices"
)

type ThanatosBuildParameter struct {
	Name          string
	Description   string
	DefaultValue  interface{}
	Choices       []string
	ParameterType agentstructs.BuildParameterType
	VerifierRegex string
	Required      bool
	ParseFunction func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error
}

// Metadata defining the Mythic payload type
var ThanatosPayload = agentstructs.PayloadType{
	// Set the name in Mythic
	Name: "thanatos",

	// Default to no file extension for payload builds
	FileExtension: "",

	// Authors
	Author: "@M_alphaaa",

	// Specifiy that the payload only supports Linux and Windows
	SupportedOS: []string{
		agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
	},

	// This payload is not a wrapper payload
	Wrapper: false,

	// Supported wrapper payloads. We don't support any yet
	CanBeWrappedByTheFollowingPayloadTypes: []string{},

	// Supports loading commands at runtime
	SupportsDynamicLoading: true,

	// Description for the payload in Mythic
	Description: "Linux and Windows agent written in Rust",

	// Has support for the HTTP and TCP C2 profiles
	SupportedC2Profiles: []string{
		"http", "tcp",
	},

	// Specify that Mythic handles encryption
	MythicEncryptsData: true,

	// Specified build steps for the agent
	BuildSteps: []agentstructs.BuildStep{},
}

var ThanatosBuildParameters = []ThanatosBuildParameter{
	// Supported build architectures. Only 32 bit (x86) and 64 bit (amd64) options
	{
		Name:         "architecture",
		Description:  "Target architecture of the agent.",
		DefaultValue: string(types.PayloadBuildParameterArchitectureAmd64),
		Choices: []string{
			string(types.PayloadBuildParameterArchitectureAmd64),
			string(types.PayloadBuildParameterArchitectureX86),
		},
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
		Required:      true,
	},

	// Option for setting the Rust build mode
	{
		Name:         "buildmode",
		Description:  "Rust build mode for compiling the agent. Useful for debugging any issues",
		DefaultValue: string(types.PayloadBuildParameterRustBuildModeRelease),
		Choices: []string{
			string(types.PayloadBuildParameterRustBuildModeRelease),
			string(types.PayloadBuildParameterRustBuildModeDebug),
		},
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
		Required:      true,
	},

	// This parameter modifies how the payload should initially execute. The options
	// are to either spawn a new thread and run the payload in the child thread while
	// the main thread exists or to fully daemonize the payload and have it run in the
	// background
	{
		Name:         "initaction",
		Description:  "Action to perform when the agent is first executed.",
		DefaultValue: string(types.PayloadBuildParameterInitActionNone),
		Choices: []string{
			string(types.PayloadBuildParameterInitActionNone),
			string(types.PayloadBuildParameterInitActionThread),
			string(types.PayloadBuildParameterInitActionFork),
		},
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
		Required:      true,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			initoption, err := pbm.BuildParameters.GetChooseOneArg(name)
			if err != nil {
				return thanatoserror.Errorf("could not get parameter: %s", err.Error())
			}

			switch types.PayloadBuildParameterInitAction(initoption) {
			case types.PayloadBuildParameterInitActionNone:
				c.Initaction = config.InitAction_NONE
			case types.PayloadBuildParameterInitActionThread:
				c.Initaction = config.InitAction_THREAD
			case types.PayloadBuildParameterInitActionFork:
				c.Initaction = config.InitAction_FORK
			default:
				return thanatoserror.Errorf("invalid init option: %s", initoption)
			}

			return nil
		},
	},

	// This determines how many times the agent should try to reconnect to Mythic if
	// there is a failed connection
	{
		Name:          "connection_retries",
		Description:   "Number of times to try and reconnect to Mythic on failed connections (use -1 for unlimited)",
		DefaultValue:  -1,
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_NUMBER,
		Required:      true,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			retries, err := pbm.BuildParameters.GetNumberArg(name)
			if err != nil {
				return thanatoserror.Errorf("could not get parameter: %s", err.Error())
			}

			retriesInt64 := int64(retries)

			// Check for overflows
			if retriesInt64 > math.MaxInt32 {
				return thanatoserror.New("value is too large. Use -1 for unlimited retries")
			}

			if retriesInt64 < -1 {
				return thanatoserror.New("value is negative. Use -1 for unlimited retries")
			}

			c.ConnectionRetries = int32(retriesInt64)
			return nil
		},
	},

	// Interval of time the agent should be active. The agent will not check in
	// outside of this interval and it will shutdown any active jobs
	{
		Name:          "working_hours",
		Description:   "Working hours for the agent. Use 24 hour time with an optional IANA time zone.",
		DefaultValue:  "00:00-23:59",
		VerifierRegex: `^[0-2][0-9]:[0-5][0-9]-[0-2][0-9]:[0-5][0-9](\s[a-zA-Z]+([a-zA-Z0-9\/_\-\+]+)?)?`,
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
		Required:      true,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			workingHoursValue, err := pbm.BuildParameters.GetStringArg(name)
			if err != nil {
				return thanatoserror.Errorf("could not get parameter: %s", err.Error())
			}

			tzSplit := strings.Split(workingHoursValue, " ")
			if len(tzSplit) > 1 {
				loc, err := time.LoadLocation(tzSplit[1])
				if err != nil {
					return thanatoserror.Errorf("failed to find timezone %s: %s", tzSplit[1], err.Error())
				}

				_, offset := time.Now().UTC().In(loc).Zone()

				c.WorkingHours = &config.WorkingHours{
					UseSystemTz: false,
					UtcOffset:   int32(offset),
				}

			} else {
				c.WorkingHours = &config.WorkingHours{
					UseSystemTz: true,
					UtcOffset:   0,
				}
			}

			workingHoursSplit := strings.Split(tzSplit[0], "-")
			if len(workingHoursSplit) == 1 {
				return thanatoserror.New("working hours value does not contain a '-' delimiter")
			}

			workingStartTime, err := workingHoursValueToDuration(workingHoursSplit[0])
			if err != nil {
				return errors.Join(thanatoserror.New("failed to parse start portion of the working hours"), err)
			}

			c.WorkingHours.Start = uint32(workingStartTime.Minutes())

			workingEndTime, err := workingHoursValueToDuration(workingHoursSplit[1])
			if err != nil {
				return errors.Join(thanatoserror.New("failed to parse end portion of the working hours"), err)
			}

			c.WorkingHours.End = uint32(workingEndTime.Minutes())

			return nil
		},
	},

	// The user can supply a list of domains the agent is allowed to execute in. The
	// domain information is retrieved before the check in and compared to this list.
	// If the domain the machine is connected to is not in this list, the agent will
	// exit. The domains, hostnames and usernames lists are 'AND'ed together. If the
	// domain is in the list but the hostname is not, the agent will not execute
	{
		Name:          "domains",
		Description:   "Limit payload execution to machines joined to one of the following domains.",
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
		Required:      false,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			domains, err := pbm.BuildParameters.GetArrayArg(name)
			if err != nil {
				return nil
			}

			c.Domains = hashList(domains)
			return nil
		},
	},

	// The user can supply a list of hosts the agent is allowed to execute on via
	// their hostnames. The domains, hostnames and usernames lists are 'AND'ed
	// together. If the machine's hostname is in the list but the domains list or
	// usernames list does not match, the agent will not execute
	{
		Name:          "hostnames",
		Description:   "Limit payload execution to machines with one of the specified hostnames.",
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
		Required:      false,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			hostnames, err := pbm.BuildParameters.GetArrayArg(name)
			if err != nil {
				return nil
			}

			c.Hostnames = hashList(hostnames)
			return nil
		},
	},

	// The user can supply a list of usernames the agent is allowed to execute as. If
	// the current user is not in the list, the agent will exist. The domains,
	// hostnames and usernames lists are 'AND'ed together. If the current username is
	// in the list but the domains list or hostnames list does not match, the agent
	// will not execute
	{
		Name:          "usernames",
		Description:   "Limit payload execution to users with one of the specified usernames.",
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
		Required:      false,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			usernames, err := pbm.BuildParameters.GetArrayArg(name)
			if err != nil {
				return nil
			}

			c.Usernames = hashList(usernames)
			return nil
		},
	},

	// This option determines whether the agent should connect to Mythic via a
	// self-signed TLS certificate
	{
		Name:          "tlsuntrusted",
		Description:   "Allow HTTPs connections to untrusted TLS certificates.",
		DefaultValue:  false,
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_BOOLEAN,
		Required:      false,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			tlsuntrusted, err := pbm.BuildParameters.GetBooleanArg(name)
			if err != nil {
				return thanatoserror.Errorf("could not get parameter: %s", err.Error())
			}

			c.Tlsuntrusted = tlsuntrusted
			return nil
		},
	},

	// An initial value for spawn to
	{
		Name:          "spawnto",
		Description:   "Initial spawnto value",
		DefaultValue:  "",
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
		Required:      false,
		ParseFunction: func(name string, c *config.Config, pbm *agentstructs.PayloadBuildMessage) error {
			spawnto, err := pbm.BuildParameters.GetStringArg(name)
			if err != nil {
				return thanatoserror.Errorf("could not get parameter: %s", err.Error())
			}

			if len(spawnto) > 0 {
				if pbm.SelectedOS == agentstructs.SUPPORTED_OS_WINDOWS {
					if !regexp.MustCompile(`^[a-zA-Z]:\\.+\.exe$`).MatchString(spawnto) {
						return thanatoserror.Errorf("spawnto value is not an absolute path to an executable")
					}
				} else if pbm.SelectedOS == agentstructs.SUPPORTED_OS_LINUX {
					if spawnto[0] != '/' {
						return thanatoserror.Errorf("spawnto value needs to be an absolute path")
					}
				} else {
					return thanatoserror.Errorf("invalid build OS value: %s", pbm.SelectedOS)
				}
			}

			c.SpawnTo = spawnto
			return nil
		},
	},

	// Name of the shared library export if building as a shared library
	{
		Name:          "libexport",
		Description:   "Shared library export name (if building as a shared library).",
		DefaultValue:  "init",
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
		Required:      false,
	},

	// The output format for the build
	{
		Name:          "output",
		Description:   "Payload output format.",
		DefaultValue:  string(types.PayloadBuildParameterOutputFormatExecutable),
		ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
		Choices: []string{
			string(types.PayloadBuildParameterOutputFormatExecutable),
			string(types.PayloadBuildParameterOutputFormatSharedLibraryInit),
			string(types.PayloadBuildParameterOutputFormatSharedLibraryExport),
			string(types.PayloadBuildParameterOutputFormatWindowsShellcode),
			string(types.PayloadBuildParameterOutputFormatSourceCode),
		},
		Required: true,
	},
}

var BuiltinCommands = []string{
	"sleep",
	"exit",
}

// Type for the handler routines when being built by Mythic
type MythicPayloadHandler struct{}

const AGENT_CODE_PATH = "agent"

// Secondary entrypoint for the payload builder. This takes in the payload build message
// and a handler which consists of a set of routines for doing long-running tasks and
// Mythic RPC calls
func BuildPayload(handler types.BuildHandler, payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	// Create the build response
	payloadBuildResponse := agentstructs.PayloadBuildResponse{
		PayloadUUID:        payloadBuildMsg.PayloadUUID,
		Success:            false,
		UpdatedCommandList: &payloadBuildMsg.CommandList,
	}

	payloadConfig, err := CreateConfig(payloadBuildMsg)
	if err != nil {
		payloadBuildResponse.BuildStdErr = errors.Join(thanatoserror.New("failed to parse payload paramters"), err).Error()
		return payloadBuildResponse
	}

	target := ""
	arch, _ := payloadBuildMsg.BuildParameters.GetStringArg("architecture")
	switch types.PayloadBuildParameterArchitecture(arch) {
	case types.PayloadBuildParameterArchitectureAmd64:
		target = "x86_64-"
	case types.PayloadBuildParameterArchitectureX86:
		target = "i686-"
	default:
		payloadBuildResponse.BuildStdErr = fmt.Sprintf("unknown architecture: %s", arch)
		return payloadBuildResponse
	}

	switch payloadBuildMsg.SelectedOS {
	case agentstructs.SUPPORTED_OS_LINUX:
		target += "unknown-linux-gnu"
	case agentstructs.SUPPORTED_OS_WINDOWS:
		target += "pc-windows-gnu"
	}

	output, _ := payloadBuildMsg.BuildParameters.GetStringArg("output")

	buildVariant := ""
	switch types.PayloadBuildParameterOutputFormat(output) {
	case types.PayloadBuildParameterOutputFormatExecutable:
		buildVariant = "binary"
	default:
		payloadBuildResponse.BuildStdErr = fmt.Sprintf("unimplemented output: %s", output)
		return payloadBuildResponse
	}

	configFile, err := os.CreateTemp("", "thanatos-config*")
	if err != nil {
		payloadBuildResponse.BuildStdErr = thanatoserror.Errorf("failed to create tempfile for config: %s", err.Error()).Error()
		return payloadBuildResponse
	}
	defer os.Remove(configFile.Name())

	if _, err := configFile.Write([]byte(payloadConfig.String())); err != nil {
		payloadBuildResponse.BuildStdErr = thanatoserror.Errorf("failed to write config to config file: %s", err.Error()).Error()
		return payloadBuildResponse
	}

	buildCommand := fmt.Sprintf(
		"env CONFIG=%s cargo build --target %s -p thanatos_%s --release",
		configFile.Name(),
		target,
		buildVariant,
	)

	payloadBuildResponse.BuildMessage += "Build Command:\n"
	payloadBuildResponse.BuildMessage += buildCommand

	// Build the payload
	payload, err := handler.RunBuildCommand(buildCommand, target, types.PayloadBuildParameterOutputFormat(output))
	if err != nil {
		payloadBuildResponse.BuildStdErr = errors.Join(thanatoserror.New("failed to build the payload"), err).Error()
		return payloadBuildResponse
	}

	payloadBuildResponse.Payload = &payload

	payloadBuildResponse.Success = true
	return payloadBuildResponse
}

// This will build the agent using the specified command string
func (handler MythicPayloadHandler) RunBuildCommand(command string, target string, output types.PayloadBuildParameterOutputFormat) ([]byte, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return []byte{}, thanatoserror.Errorf("failed to get the current working directory: %s", err.Error())
	}

	agentCodePath := fmt.Sprintf("%s/%s", cwd, AGENT_CODE_PATH)

	cmd := exec.Command("/bin/bash", "-c", command)
	cmd.Dir = agentCodePath

	cmdOutput, err := cmd.CombinedOutput()
	if err != nil {
		errorMsg := thanatoserror.Errorf("failed to build the agent: %s", err.Error())
		return []byte{}, errors.Join(thanatoserror.Errorf("output for command '/bin/bash -c %s:\n%s", command, string(cmdOutput)), errorMsg)
	}

	outpath := fmt.Sprintf("%s/target/%s/release", agentCodePath, target)

	filename := ""
	if strings.Contains(target, "-unknown-linux-gnu") {
		switch output {
		case types.PayloadBuildParameterOutputFormatExecutable:
			filename = "thanatos_binary"
		default:
			filename = "libthanatos_cdylib.so"
		}
	} else if strings.Contains(target, "-pc-windows-gnu") {
		switch output {
		case types.PayloadBuildParameterOutputFormatExecutable:
			filename = "thanatos_binary.exe"
		default:
			filename = "thanatos_cdylib.dll"
		}
	} else {
		return []byte{}, thanatoserror.New("invalid target")
	}

	payload, err := os.ReadFile(fmt.Sprintf("%s/%s", outpath, filename))
	if err != nil {
		return []byte{}, thanatoserror.Errorf("failed to open the built payload: %s", err.Error())
	}

	return payload, nil
}

func CreateConfig(buildMsg agentstructs.PayloadBuildMessage) (*config.Config, error) {
	payloadUuid, err := uuid.Parse(buildMsg.PayloadUUID)
	if err != nil {
		return nil, thanatoserror.Errorf("failed to parse payload uuid: %s", err.Error())
	}

	resultConfig := &config.Config{
		Uuid: payloadUuid[:],
	}

	if err := parsePayloadParameters(resultConfig, buildMsg); err != nil {
		return nil, errors.Join(thanatoserror.New("failed to parse payload build parameters"), err)
	}

	egressEnabled := false
	p2pEnabled := false

	for _, profile := range buildMsg.C2Profiles {
		switch profile.Name {
		case "http":
			egressEnabled = true
			if err := ParseHttpProfile(resultConfig, profile); err != nil {
				return nil, errors.Join(thanatoserror.New("failed to parse http profile parameters"), err)
			}
		case "tcp":
			p2pEnabled = true
			return nil, thanatoserror.New("tcp profile unimplemented")
		}
	}

	if egressEnabled && p2pEnabled {
		return nil, thanatoserror.Errorf("cannot mix egress and p2p C2 profiles")
	}

	return resultConfig, nil
}

func parsePayloadParameters(resultConfig *config.Config, buildMsg agentstructs.PayloadBuildMessage) error {
	for name := range buildMsg.BuildParameters.Parameters {
		paramIndex := slices.IndexFunc(ThanatosBuildParameters, func(param ThanatosBuildParameter) bool {
			return param.Name == name
		})

		if paramIndex == -1 {
			return thanatoserror.Errorf("failed to find build parameter %s", name)
		}

		if parseFn := ThanatosBuildParameters[paramIndex].ParseFunction; parseFn != nil {
			if err := parseFn(name, resultConfig, &buildMsg); err != nil {
				return errors.Join(thanatoserror.Errorf("failed to parse %s build parameter", name), err)
			}
		}
	}
	return nil
}

func hashList(values []string) []byte {
	result := []byte{}

	for _, val := range values {
		h := sha256.Sum256([]byte(strings.ToLower(val)))
		result = append(result, h[:]...)
	}

	return result
}

func workingHoursValueToDuration(value string) (time.Duration, error) {
	parsedDuration := time.Duration(0)

	// Split the duration into separate hour and minute values
	stringSplit := strings.Split(value, ":")
	if len(stringSplit) == 1 {
		return parsedDuration, thanatoserror.New("did not find a ':' delimiter in the time value")
	} else if len(stringSplit) != 2 {
		return parsedDuration, thanatoserror.New("time value is malformed")
	}

	// Convert the hour portion to an integer
	hour, err := strconv.Atoi(stringSplit[0])
	if err != nil {
		return parsedDuration, thanatoserror.New("failed to parse the hours portion of the time value")
	}

	// Validate the hour portion
	if hour > 23 {
		return parsedDuration, thanatoserror.New("hour portion cannot be greater than 23")
	} else if hour < 0 {
		return parsedDuration, thanatoserror.New("hour portion is negative")
	}

	// Convert the minute portion to an integer
	minute, err := strconv.Atoi(stringSplit[1])
	if err != nil {
		return parsedDuration, thanatoserror.New("failed to parse the minutes potion of the time value")
	}

	// Validate the minute portion
	if minute > 60 {
		return parsedDuration, thanatoserror.New("minute portion cannot be greater than 60")
	} else if minute < 0 {
		return parsedDuration, thanatoserror.New("minute portion is negative")
	}

	// Convert the hour period to seconds
	hour = hour * 60 * 60

	// Convert the minute period to seconds
	minute = minute * 60

	// Get the duration in total seconds
	durationSeconds := float64(hour) + float64(minute)

	// Convert the seconds to nano seconds and create a time.Duration
	parsedDuration = time.Duration(durationSeconds * float64(time.Second))
	return parsedDuration, nil

}

// Main entrypoint when Mythic executes the payload builder
func mythicBuildPayloadFunction(payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	return BuildPayload(MythicPayloadHandler{}, payloadBuildMsg)
}

// This updates the current build step in Mythic
func (handler MythicPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return mythicrpc.SendMythicRPCPayloadUpdateBuildStep(input)
}

// Initializes the payload build routines in Mythic
func Initialize() {
	for _, param := range ThanatosBuildParameters {
		ThanatosPayload.BuildParameters = append(ThanatosPayload.BuildParameters, agentstructs.BuildParameter{
			Name:          param.Name,
			Description:   param.Description,
			DefaultValue:  param.DefaultValue,
			Choices:       param.Choices,
			ParameterType: param.ParameterType,
			VerifierRegex: param.VerifierRegex,
			Required:      param.Required,
		})
	}

	agentstructs.AllPayloadData.Get("thanatos").AddPayloadDefinition(ThanatosPayload)
	agentstructs.AllPayloadData.Get("thanatos").AddIcon(filepath.Join(".", "assets", "thanatos.svg"))
	agentstructs.AllPayloadData.Get("thanatos").AddBuildFunction(mythicBuildPayloadFunction)

}
