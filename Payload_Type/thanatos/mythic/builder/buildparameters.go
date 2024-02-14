// Handles parsing all of the payload parameters
package builder

import (
	"errors"
	thanatoserror "thanatos/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Strongly type struct containing all of the build parameters from Mythic
type ParsedBuildParameters struct {
	// Supported architectures of the agent
	Architecture PayloadBuildParameterArchitecture

	// Agent's initial exection parameters
	InitOptions PayloadBuildParameterInitOptions

	// Number of tries to reconnect to Mythic on failed connections
	ConnectionRetries uint32

	// Library for doing crypto
	CryptoLib PayloadBuildParameterCryptoLibrary

	// Working hours
	WorkingHours ParsedWorkingHours

	// List of domains for execution guardrails
	DomainList []string

	// List of hostnames for execution guardrails
	HostnameList []string

	// List of usernames for execution guardrails
	UsernameList []string

	// Options for static linking
	StaticOptions []PayloadBuildParameterStaticOption

	// Whether the agent should connect to self signed TLS certificates
	TlsUntrusted bool

	// Initial spawnto value
	SpawnTo string

	// Export name for shared libraries
	LibExportName string

	// Output format
	Output PayloadBuildParameterOutputFormat
}

// Parses the build parameters from Mythic to a strongly typed structure
func parsePayloadBuildParameters(buildMessage agentstructs.PayloadBuildMessage) (ParsedBuildParameters, error) {
	const errorFormatStr string = "failed to get the '%s' value from the payload build parameters: %s"

	parameters := buildMessage.BuildParameters

	parsedParameters := ParsedBuildParameters{}

	architecture, err := parameters.GetStringArg("architecture")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "architecture", err.Error())
	}

	parsedParameters.Architecture = PayloadBuildParameterArchitecture(architecture)

	initOptions, err := parameters.GetStringArg("initoptions")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "initoptions", err.Error())
	}

	if initOptions == string(PayloadBuildParameterInitOptionFork) && buildMessage.SelectedOS == agentstructs.SUPPORTED_OS_WINDOWS {
		return parsedParameters, thanatoserror.New("cannot build a Windows payload with the fork initial execution option")
	}

	parsedParameters.InitOptions = PayloadBuildParameterInitOptions(initOptions)

	connectionRetries, err := parameters.GetNumberArg("connection_retries")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "connection_retries", err.Error())
	}

	if connectionRetries <= 0 {
		return parsedParameters, thanatoserror.New("connection_retries value is <= 0")
	}

	parsedParameters.ConnectionRetries = uint32(connectionRetries)

	cryptoLib, err := parameters.GetStringArg("cryptolib")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "cryptolib", err.Error())
	}

	parsedParameters.CryptoLib = PayloadBuildParameterCryptoLibrary(cryptoLib)

	workingHoursStr, err := parameters.GetStringArg("working_hours")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "working_hours", err.Error())
	}

	workingHours, err := parseWorkingHours(workingHoursStr)
	if err != nil {
		return parsedParameters, errors.Join(thanatoserror.New("failed to parse the payload's working hours"), err)
	}

	if workingHours.StartTime >= workingHours.EndTime {
		return parsedParameters, thanatoserror.New("working hours start time is after the working hours end time")
	}

	parsedParameters.WorkingHours = workingHours

	if domainsList, err := parameters.GetArrayArg("domains"); err == nil {
		parsedParameters.DomainList = domainsList
	} else {
		parsedParameters.DomainList = []string{}
	}

	if hostnamesList, err := parameters.GetArrayArg("hostnames"); err == nil {
		parsedParameters.HostnameList = hostnamesList
	} else {
		parsedParameters.HostnameList = []string{}
	}

	if usernamesList, err := parameters.GetArrayArg("usernames"); err == nil {
		parsedParameters.UsernameList = usernamesList
	} else {
		parsedParameters.UsernameList = []string{}
	}

	staticOptions, err := parameters.GetArrayArg("static")
	if err == nil {
		for _, option := range staticOptions {
			parsedParameters.StaticOptions = append(parsedParameters.StaticOptions, PayloadBuildParameterStaticOption(option))
		}
	} else {
		parsedParameters.StaticOptions = []PayloadBuildParameterStaticOption{}
	}

	tlsuntrusted, err := parameters.GetBooleanArg("tlsuntrusted")
	if err == nil {
		parsedParameters.TlsUntrusted = tlsuntrusted
	} else {
		parsedParameters.TlsUntrusted = false
	}

	spawnto, err := parameters.GetStringArg("spawnto")
	if err == nil {
		parsedParameters.SpawnTo = spawnto
	} else {
		parsedParameters.SpawnTo = ""
	}

	parsedParameters.LibExportName, _ = parameters.GetStringArg("libexport")

	output, err := parameters.GetStringArg("output")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "output", err.Error())
	}

	parsedParameters.Output = PayloadBuildParameterOutputFormat(output)

	return parsedParameters, nil
}
