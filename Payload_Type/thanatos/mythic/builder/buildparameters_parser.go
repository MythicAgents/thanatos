// Handles parsing all of the payload parameters
package builder

import (
	"errors"
	builderrors "thanatos/builder/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Strongly type struct containing all of the build parameters from Mythic
type ParsedBuildParameters struct {
	// Supported architectures of the agent
	Architecture PayloadBuildParameterArchitecture

	// Agent's initial exection parameters
	InitOptions PayloadBuildParameterInitOptions

	// Number of tries to reconnect to Mythic on failed connections
	ConnectionRetries int

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
	TlsSelfSigned bool

	// Initial spawnto value
	SpawnTo string

	// Output format for the agent
	Output PayloadBuildParameterOutputFormat
}

// Parses the build parameters from Mythic to a strongly typed structure
func parsePayloadBuildParameters(buildMessage agentstructs.PayloadBuildMessage) (ParsedBuildParameters, error) {
	const errorFormatStr string = "failed to get the '%s' value from the payload build parameters: %s"

	configuredOS := buildMessage.SelectedOS
	_ = configuredOS
	parameters := buildMessage.BuildParameters

	parsedParameters := ParsedBuildParameters{}

	architecture, err := parameters.GetStringArg("architecture")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "architecture", err.Error())
	}

	if arch := NewPayloadBuildParameterArchitecture(architecture); arch != nil {
		parsedParameters.Architecture = *arch
	} else {
		return parsedParameters, builderrors.Errorf("invalid architecture string value: %s", architecture)
	}

	initOptions, err := parameters.GetStringArg("initoptions")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "initoptions", err.Error())
	}

	parsedParameters.InitOptions = PayloadBuildParameterInitOptions(initOptions)

	connectionRetries, err := parameters.GetNumberArg("connection_retries")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "connection_retries", err.Error())
	}

	if connectionRetries <= 0 {
		return parsedParameters, builderrors.New("connection_retries value is <= 0")
	}

	parsedParameters.ConnectionRetries = int(connectionRetries)

	cryptoLib, err := parameters.GetStringArg("cryptolib")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "cryptolib", err.Error())
	}

	parsedParameters.CryptoLib = PayloadBuildParameterCryptoLibrary(cryptoLib)

	workingHoursStr, err := parameters.GetStringArg("working_hours")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "working_hours", err.Error())
	}

	workingHours, err := parseWorkingHours(workingHoursStr)
	if err != nil {
		return parsedParameters, errors.Join(builderrors.New("failed to parse the payload's working hours"), err)
	}

	if workingHours.StartTime >= workingHours.EndTime {
		return parsedParameters, builderrors.New("working hours start time is after the working hours end time")
	}

	parsedParameters.WorkingHours = workingHours

	if domainsList, err := parameters.GetArrayArg("domains"); err == nil {
		parsedParameters.DomainList = domainsList
	} else {
		parsedParameters.DomainList = make([]string, 0)
	}

	if hostnamesList, err := parameters.GetArrayArg("hostnames"); err == nil {
		parsedParameters.HostnameList = hostnamesList
	} else {
		parsedParameters.HostnameList = make([]string, 0)
	}

	if usernamesList, err := parameters.GetArrayArg("usernames"); err == nil {
		parsedParameters.UsernameList = usernamesList
	} else {
		parsedParameters.UsernameList = make([]string, 0)
	}

	staticOptions, err := parameters.GetArrayArg("static")
	if err == nil {
		for _, option := range staticOptions {
			parsedParameters.StaticOptions = append(parsedParameters.StaticOptions, PayloadBuildParameterStaticOption(option))
		}
	} else {
		parsedParameters.StaticOptions = make([]PayloadBuildParameterStaticOption, 0)
	}

	tlsselfsigned, err := parameters.GetBooleanArg("tlsselfsigned")
	if err == nil {
		parsedParameters.TlsSelfSigned = tlsselfsigned
	} else {
		parsedParameters.TlsSelfSigned = false
	}

	spawnto, err := parameters.GetStringArg("spawnto")
	if err == nil {
		parsedParameters.SpawnTo = spawnto
	} else {
		parsedParameters.SpawnTo = ""
	}

	output, err := parameters.GetStringArg("output")
	if err != nil {
		return parsedParameters, builderrors.Errorf(errorFormatStr, "output", err.Error())
	}

	parsedParameters.Output = PayloadBuildParameterOutputFormat(output)

	return parsedParameters, nil
}
