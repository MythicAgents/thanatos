package builder

import (
	"errors"
	"fmt"
	"path/filepath"
	"sync"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Interface for building a payload
type PayloadBuilder interface {
	// Method which takes in the raw command for building the agent and returns the contents
	// of the built payload for Mythic
	Build(command string) ([]byte, error)
}

// Strongly type struct containing all of the build parameters from Mythic
type ParsedBuildParameters struct {
	Architecture PayloadBuildParameterArchitecture
	InitOptions  PayloadBuildParameterInitOptions
	CryptoLib    PayloadBuildParameterCryptoLibrary
	WorkingHours struct {
		StartTime time.Duration
		EndTime   time.Duration
	}

	DomainList    []string
	HostnameList  []string
	UsernameList  []string
	StaticOptions []PayloadBuildParameterStaticOption
	TlsSelfSigned bool
	SpawnTo       string
	Output        PayloadBuildParameterOutputFormat
}

// Metadata defining the Mythic payload
var payloadDefinition = agentstructs.PayloadType{
	Name:          "thanatos",
	FileExtension: "",
	Author:        "@M_alphaaa",
	SupportedOS: []string{
		agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
	},
	Wrapper:                                false,
	CanBeWrappedByTheFollowingPayloadTypes: []string{},
	SupportsDynamicLoading:                 true,
	Description:                            "Linux and Windows agent written in Rust",
	SupportedC2Profiles: []string{
		"http", "tcp",
	},
	MythicEncryptsData: true,

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
	BuildSteps: []agentstructs.BuildStep{},
}

// Mutex for restricting parallel builds. The rust compiler likes to use a lot of CPU resources.
// This can be problematic when the payload builder is run on the same system Mythic is running on.
// To prevent payload builds from accidentally DOSing the Mythic server, only allow sequential builds.
// Parallel build support may be added back in the future.
var payloadBuildLock sync.Mutex

type PayloadCommandBuilder struct{}

func (builder PayloadCommandBuilder) Build(command string) ([]byte, error) {
	return make([]byte, 0), nil
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
		return parsedParameters, errors.New(fmt.Sprintf("Invalid architecture string value: %s", architecture))
	}

	initOptions, err := parameters.GetStringArg("initoptions")
	if err != nil {
		return parsedParameters, err
	}

	parsedParameters.InitOptions = PayloadBuildParameterInitOptions(initOptions)

	return parsedParameters, nil
}

// Function which builds the payload with a configured payload builder
func buildPayload(payloadBuildMsg agentstructs.PayloadBuildMessage, builder PayloadBuilder) agentstructs.PayloadBuildResponse {
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
	builder := PayloadCommandBuilder{}
	return buildPayload(payloadBuildMsg, &builder)
}

// Initializes the agent in Mythic
func Initialize() {
	agentstructs.AllPayloadData.Get("thanatos").AddPayloadDefinition(payloadDefinition)
	agentstructs.AllPayloadData.Get("thanatos").AddIcon(filepath.Join(".", "thanatos", "mythic", "assets", "thanatos.svg"))
	agentstructs.AllPayloadData.Get("thanatos").AddBuildFunction(mythicBuildRoutine)
}
