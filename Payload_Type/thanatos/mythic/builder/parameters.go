// Payload build parameters
package builder

import (
	"crypto/sha256"
	"errors"
	"math"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/MythicAgents/thanatos/builder/types"
	thanatoserror "github.com/MythicAgents/thanatos/errors"
	"github.com/MythicAgents/thanatos/pb/config"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
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
