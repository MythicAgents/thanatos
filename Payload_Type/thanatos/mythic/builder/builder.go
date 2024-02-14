// Main builder entrypoint. This is where the payload builder is defined and implemented
package builder

import (
	"errors"
	"os"
	"path/filepath"
	thanatoserror "thanatos/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Metadata defining the Mythic payload type
var payloadDefinition = agentstructs.PayloadType{
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

	// Build parameters of the payload
	BuildParameters: []agentstructs.BuildParameter{
		// Supported build architectures. Only 32 bit (x86) and 64 bit (amd64) options
		{
			Name:         "architecture",
			Description:  "Architecture of the agent",
			DefaultValue: string(PayloadBuildParameterArchitectureAmd64),
			Choices: []string{
				string(PayloadBuildParameterArchitectureAmd64),
				string(PayloadBuildParameterArchitectureX86),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		// This parameter modifies how the payload should initially execute. The options
		// are to either spawn a new thread and run the payload in the child thread while
		// the main thread exists or to fully daemonize the payload and have it run in the
		// background
		{
			Name:         "initoptions",
			Description:  "Initial execution option",
			DefaultValue: string(PayloadBuildParameterInitOptionNone),
			Choices: []string{
				string(PayloadBuildParameterInitOptionNone),
				string(PayloadBuildParameterInitOptionSpawnThread),
				string(PayloadBuildParameterInitOptionFork),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		// This determines how many times the agent should try to reconnect to Mythic if
		// there is a failed connection
		{
			Name:          "connection_retries",
			Description:   "Number of times to try and reconnect to Mythic on failed connections",
			DefaultValue:  1,
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_NUMBER,
			Required:      true,
		},

		// This affects what library is used for doing any sort of cryptography. The
		// internal library uses statically linked pure Rust crypto routines. The system
		// library will use openssl on Linux and Windows CNG libraries
		{
			Name:         "cryptolib",
			Description:  "Library to use for doing cryptographic functions",
			DefaultValue: string(PayloadBuildParameterCryptoLibrarySystem),
			Choices: []string{
				string(PayloadBuildParameterCryptoLibraryInternal),
				string(PayloadBuildParameterCryptoLibrarySystem),
			},
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Required:      true,
		},

		// Interval of time the agent should be active. The agent will not check in
		// outside of this interval and it will shutdown any active jobs
		{
			Name:          "working_hours",
			Description:   "Working hours for the agent (use 24 hour UTC time)",
			DefaultValue:  "00:00-23:59",
			VerifierRegex: "^[0-2][0-9]:[0-5][0-9]-[0-2][0-9]:[0-5][0-9]",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
			Required:      true,
		},

		// The user can supply a list of domains the agent is allowed to execute in. The
		// domain information is retrieved before the check in and compared to this list.
		// If the domain the machine is connected to is not in this list, the agent will
		// exit. The domains, hostnames and usernames lists are 'AND'ed together. If the
		// domain is in the list but the hostname is not, the agent will not execute
		{
			Name:          "domains",
			Description:   "Limit payload execution to machines joined to one of the following domains",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		// The user can supply a list of hosts the agent is allowed to execute on via
		// their hostnames. The domains, hostnames and usernames lists are 'AND'ed
		// together. If the machine's hostname is in the list but the domains list or
		// usernames list does not match, the agent will not execute
		{
			Name:          "hostnames",
			Description:   "Limit payload execution to machines with one of the specified hostnames",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		// The user can supply a list of usernames the agent is allowed to execute as. If
		// the current user is not in the list, the agent will exist. The domains,
		// hostnames and usernames lists are 'AND'ed together. If the current username is
		// in the list but the domains list or hostnames list does not match, the agent
		// will not execute
		{
			Name:          "usernames",
			Description:   "Limit payload execution to users with one of the specified usernames",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_ARRAY,
			Required:      false,
		},

		// List defining what libraries should be statically linked to
		{
			Name:          "static",
			Description:   "Statically link the following libraries (Linux only)",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_MULTIPLE,
			Choices: []string{
				string(PayloadBuildParameterStaticOptionOpenSSL),
				string(PayloadBuildParameterStaticOptionLibCurl),
			},
			Required: false,
		},

		// This option determines whether the agent should connect to Mythic via a
		// self-signed TLS certificate
		{
			Name:          "tlsuntrusted",
			Description:   "Allow HTTPs connections to untrusted TLS certificates",
			DefaultValue:  false,
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_BOOLEAN,
			Required:      false,
		},

		// An initial value for spawn to
		{
			Name:          "spawnto",
			Description:   "Initial spawnto value",
			DefaultValue:  "",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
			Required:      false,
		},

		// Name of the shared library export if building as a shared library
		{
			Name:          "libexport",
			Description:   "Shared library export name (if building as a shared library)",
			DefaultValue:  "init",
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
			Required:      false,
		},

		// The output format for the build
		{
			Name:          "output",
			Description:   "Payload output format",
			DefaultValue:  string(PayloadBuildParameterOutputFormatExecutable),
			ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
			Choices: []string{
				string(PayloadBuildParameterOutputFormatExecutable),
				string(PayloadBuildParameterOutputFormatSharedLibraryInit),
				string(PayloadBuildParameterOutputFormatSharedLibraryExport),
				string(PayloadBuildParameterOutputFormatReflectiveSharedLibrary),
				string(PayloadBuildParameterOutputFormatWindowsShellcode),
				string(PayloadBuildParameterOutputFormatSourceCode),
			},
			Required: true,
		},
	},

	// Specified build steps for the agent
	BuildSteps: []agentstructs.BuildStep{},
}

var builtinCommands = []string{
	"sleep",
	"exit",
}

// Secondary entrypoint for the payload builder. This takes in the payload build message
// and a handler which consists of a set of routines for doing long-running tasks and
// Mythic RPC calls
func BuildPayload(handler BuildHandler, payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	// Create the build response
	payloadBuildResponse := agentstructs.PayloadBuildResponse{
		PayloadUUID:        payloadBuildMsg.PayloadUUID,
		Success:            false,
		UpdatedCommandList: &payloadBuildMsg.CommandList,
	}

	// Parse all of the payload parameters
	parameters, err := parsePayloadParameters(payloadBuildMsg)
	if err != nil {
		payloadBuildResponse.BuildStdErr = errors.Join(thanatoserror.New("failed to parse the payload parameters"), err).Error()
		return payloadBuildResponse
	}

	payloadConfig := createConfig(parameters)

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

	target := ""
	switch parameters.BuildParameters.Architecture {
	case PayloadBuildParameterArchitectureAmd64:
		target = "x86_64-"
	case PayloadBuildParameterArchitectureX86:
		target = "i686-"
	}

	switch payloadBuildMsg.SelectedOS {
	case agentstructs.SUPPORTED_OS_LINUX:
		target += "unknown-linux-gnu"
	case agentstructs.SUPPORTED_OS_WINDOWS:
		target += "pc-windows-gnu"
	}

	buildCommand := FormulateBuildCommand(target, configFile.Name(), parameters)

	payloadBuildResponse.BuildMessage += "Build Command:\n"
	payloadBuildResponse.BuildMessage += buildCommand

	// Build the payload
	payload, err := handler.Build(target, parameters, buildCommand)
	if err != nil {
		payloadBuildResponse.BuildStdErr = errors.Join(thanatoserror.New("failed to build the payload"), err).Error()
		return payloadBuildResponse
	}

	payloadBuildResponse.Payload = &payload

	payloadBuildResponse.Success = true
	return payloadBuildResponse
}

// Main entrypoint when Mythic executes the payload builder
func mythicBuildPayloadFunction(payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	return BuildPayload(MythicPayloadHandler{}, payloadBuildMsg)
}

// Initializes the payload build routines in Mythic
func Initialize() {
	agentstructs.AllPayloadData.Get("thanatos").AddPayloadDefinition(payloadDefinition)
	agentstructs.AllPayloadData.Get("thanatos").AddIcon(filepath.Join(".", "assets", "thanatos.svg"))
	agentstructs.AllPayloadData.Get("thanatos").AddBuildFunction(mythicBuildPayloadFunction)

}
