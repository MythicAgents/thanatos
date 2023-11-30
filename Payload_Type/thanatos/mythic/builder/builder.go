package builder

import (
	"errors"
	"fmt"
	"math"
	"path/filepath"
	"strconv"
	"strings"
	_ "sync"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

// Strongly type struct containing all of the build parameters from Mythic
type ParsedBuildParameters struct {
	// Supported architectures of the agent
	Architecture PayloadBuildParameterArchitecture

	// Agent's initial exection parameters
	InitOptions PayloadBuildParameterInitOptions

	// Number of tries to reconnect to Mythic on failed connections
	ConnectionRetries float64

	// Library for doing crypto
	CryptoLib PayloadBuildParameterCryptoLibrary

	// Working hours
	WorkingHours struct {
		// Working hour start time
		StartTime time.Duration

		// Working hour end time
		EndTime time.Duration
	}

	// List of domains for execution guardrails
	DomainList []string

	// List of hostnames for execution guardrails
	HostnameList []string

	// List of usernames for execution guardrails
	UsernameList []string

	// Options for static linking
	StaticOptions []PayloadBuildParameterStaticOption

	// Whether the agent should connect to self signed TLS certificates
	TlsSelfSigned bool

	// Initial spawnto value
	SpawnTo string

	// Output format for the agent
	Output PayloadBuildParameterOutputFormat
}

// Metadata defining the Mythic payload
var payloadDefinition = agentstructs.PayloadType{
	// Payload name
	Name: "thanatos",

	// Default file extension
	FileExtension: "",

	// Authors
	Author: "@M_alphaaa",

	// Supports OSs
	SupportedOS: []string{
		agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
	},

	// Whether this is a wrapper payload
	Wrapper: false,

	// Supported wrapper payloads
	CanBeWrappedByTheFollowingPayloadTypes: []string{},

	// Supports loading commands at runtime
	SupportsDynamicLoading: true,

	// Payload description
	Description: "Linux and Windows agent written in Rust",

	// C2 profiles which can be compiled into the agent
	SupportedC2Profiles: []string{
		"http", "tcp",
	},

	// Where encryption is handled
	MythicEncryptsData: true,

	// Build parameters of the payload
	BuildParameters: []agentstructs.BuildParameter{
		{
			Name:         "architecture",
			Description:  "Architecture of the agent",
			DefaultValue: "amd64",
			Choices: []string{
				string(PayloadBuildParameterArchitectureAmd64),
				string(PayloadBuildParameterArchitectureX86),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		{
			Name:         "initoptions",
			Description:  "Initial execution option",
			DefaultValue: string(PayloadBuildParameterInitOptionNone),
			Choices: []string{
				string(PayloadBuildParameterInitOptionNone),
				string(PayloadBuildParameterInitOptionSpawnThread),
				string(PayloadBuildParameterInitOptionDaemonize),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		{
			Name:          "connection_retries",
			Description:   "Number of times to try and reconnect to Mythic",
			DefaultValue:  1,
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_NUMBER,
			Required:      true,
		},

		{
			Name:         "cryptolib",
			Description:  "Library to use for doing crypto routines",
			DefaultValue: string(PayloadBuildParameterCryptoLibrarySystem),
			Choices: []string{
				string(PayloadBuildParameterCryptoLibraryInternal),
				string(PayloadBuildParameterCryptoLibrarySystem),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		{
			Name:          "working_hours",
			Description:   "Working hours for the agent (use 24 hour time)",
			DefaultValue:  "00:00-23:59",
			VerifierRegex: "^[0-2][0-9]:[0-5][0-9]-[0-2][0-9]:[0-5][0-9]",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
			Required:      true,
		},

		{
			Name:          "domains",
			Description:   "Limit payload execution to machines joined to one of the following domains",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		{
			Name:          "hostnames",
			Description:   "Limit payload execution to machines with one of the specified hostnames",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		{
			Name:          "usernames",
			Description:   "Limit payload execution to users with one of the specified usernames",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		{
			Name:        "static",
			Description: "Libraries to statically link to (Linux only)",
			Choices: []string{
				string(PayloadBuildParameterStaticOptionOpenSSL),
				string(PayloadBuildParameterStaticOptionLibCurl),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_MULTIPLE,
			Required:      false,
		},

		{
			Name:          "tlsselfsigned",
			Description:   "Allow HTTPs connections to self-signed TLS certificates",
			DefaultValue:  false,
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_BOOLEAN,
			Required:      false,
		},

		{
			Name:          "spawnto",
			Description:   "Initial spawnto value",
			DefaultValue:  "",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
			Required:      false,
		},

		{
			Name:          "output",
			Description:   "Payload output format",
			DefaultValue:  string(PayloadBuildParameterOutputFormatExecutable),
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Choices: []string{
				string(PayloadBuildParameterOutputFormatExecutable),
				string(PayloadBuildParameterOutputFormatSharedLibrary),
				string(PayloadBuildParameterOutputFormatSharedLibraryInit),
				string(PayloadBuildParameterOutputFormatWindowsShellcode),
			},
			Required: true,
		},
	},

	BuildSteps: []agentstructs.BuildStep{
		{
			Name:        "Installing Rust Target",
			Description: "Installing the reqruied Rust target for the paylod build",
		},

		{
			Name:        "Building",
			Description: "Building the payload",
		},
	},
}

// Mutex for restricting parallel builds. The rust compiler likes to use a lot of CPU resources.
// This can be problematic when the payload builder is run on the same system Mythic is running on.
// To prevent payload builds from accidentally DOSing the Mythic server, only allow sequential builds.
// Parallel build support may be added back in the future.
//var payloadBuildLock sync.Mutex

// Type for the handler routines when being built by Mythic
type MythicPayloadHandler struct{}

// Implementation for when the builder needs to build the agent
func (handler MythicPayloadHandler) Build(command string) ([]byte, error) {
	return make([]byte, 0), nil
}

// Implementation for installing a Rust target
func (handler MythicPayloadHandler) InstallTarget(target string) error {
	return nil
}

// Implementation for updating the current build step
func (handler MythicPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return mythicrpc.SendMythicRPCPayloadUpdateBuildStep(input)
}

// Converts a singular working hours value '01:30' to a duration
func workingHoursValueToDuration(value string) (time.Duration, error) {
	parsedDuration := time.Duration(0)

	// Split the duration into separate hour and minute values
	stringSplit := strings.Split(value, ":")
	if len(stringSplit) == 1 {
		return parsedDuration, errors.New("did not find a ':' delimiter in the working hour time")
	} else if len(stringSplit) != 2 {
		return parsedDuration, errors.New("working hour time is malformed")
	}

	// Convert the hour portion to an integer
	hour, err := strconv.Atoi(stringSplit[0])
	if err != nil {
		return parsedDuration, errors.New("failed to parse the hours portion of the working hours")
	}

	// Validate the hour portion
	if hour > 23 {
		return parsedDuration, errors.New("hour portion is greater than 23")
	} else if hour < 0 {
		return parsedDuration, errors.New("hour portion is negative")
	}

	// Convert the minute portion to an integer
	minute, err := strconv.Atoi(stringSplit[1])
	if err != nil {
		return parsedDuration, errors.New("failed to parse the minutes potion of the working hours")
	}

	// Validate the minute portion
	if minute > 60 {
		return parsedDuration, errors.New("minute portion is greater than 60")
	} else if minute < 0 {
		return parsedDuration, errors.New("minute portion is negative")
	}

	// Convert the hour period to seconds
	hour = hour * 60 * 60

	// Convert the minute period to seconds
	minute = minute * 60

	// Get the duration in total seconds
	durationSeconds := float64(hour) + float64(minute)

	// Convert the seconds to nano seconds and create a time.Duration
	parsedDuration = time.Duration(durationSeconds * math.Pow(10, 9))

	return parsedDuration, nil
}

// Parses the working hours '00:00-23:00' format
func parseWorkingHours(workingHours string) (time.Duration, time.Duration, error) {
	workingStart := time.Duration(0)
	workingEnd := time.Duration(0)

	workingHoursSplit := strings.Split(workingHours, "-")
	if len(workingHoursSplit) == 1 {
		return workingStart, workingEnd, errors.New("working hours value does not contain a '-' delimiter")
	}

	workingStart, err := workingHoursValueToDuration(workingHoursSplit[0])
	if err != nil {
		return workingStart, workingEnd, fmt.Errorf("failed to parse the start portion for the working hours: %s", err.Error())
	}

	workingEnd, err = workingHoursValueToDuration(workingHoursSplit[1])
	if err != nil {
		return workingStart, workingEnd, fmt.Errorf("failed to parse the end portion for the working hours: %s", err.Error())
	}

	return workingStart, workingEnd, nil
}

// Parses the build parameters from Mythic to a strongly typed structure
func parseBuildParameters(buildMessage *agentstructs.PayloadBuildMessage) (ParsedBuildParameters, error) {
	configuredOS := buildMessage.SelectedOS
	_ = configuredOS
	parameters := buildMessage.BuildParameters

	parsedParameters := ParsedBuildParameters{}

	architecture, err := parameters.GetStringArg("architecture")
	if err != nil {
		return parsedParameters, err
	}

	if arch := NewPayloadBuildParameterArchitecture(architecture); arch != nil {
		parsedParameters.Architecture = *arch
	} else {
		return parsedParameters, fmt.Errorf("invalid architecture string value: %s", architecture)
	}

	initOptions, err := parameters.GetStringArg("initoptions")
	if err != nil {
		return parsedParameters, err
	}

	parsedParameters.InitOptions = PayloadBuildParameterInitOptions(initOptions)

	connectionRetries, err := parameters.GetNumberArg("connection_retries")
	if err != nil {
		return parsedParameters, err
	}

	if connectionRetries <= 0 {
		return parsedParameters, errors.New("connection retries is <= 0")
	}

	parsedParameters.ConnectionRetries = connectionRetries

	workingHours, err := parameters.GetStringArg("working_hours")
	if err != nil {
		return parsedParameters, err
	}

	_ = workingHours

	return parsedParameters, nil
}

// Function which builds the payload with a configured payload builder
func buildPayload(payloadBuildMsg agentstructs.PayloadBuildMessage, handler BuildHandler) agentstructs.PayloadBuildResponse {
	payloadBuildResponse := agentstructs.PayloadBuildResponse{
		PayloadUUID:        payloadBuildMsg.PayloadUUID,
		Success:            false,
		UpdatedCommandList: &payloadBuildMsg.CommandList,
	}

	parameters, err := parseBuildParameters(&payloadBuildMsg)
	if err != nil {
		payloadBuildResponse.BuildStdErr = err.Error()
	}

	_ = parameters

	return payloadBuildResponse
}

// Routine invoked when Mythic requests a new payload
func mythicBuildRoutine(payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	handler := MythicPayloadHandler{}
	return buildPayload(payloadBuildMsg, &handler)
}

// Initializes the agent in Mythic
func Initialize() {
	agentstructs.AllPayloadData.Get("thanatos").AddPayloadDefinition(payloadDefinition)
	agentstructs.AllPayloadData.Get("thanatos").AddIcon(filepath.Join(".", "thanatos", "mythic", "assets", "thanatos.svg"))
	agentstructs.AllPayloadData.Get("thanatos").AddBuildFunction(mythicBuildRoutine)
}
