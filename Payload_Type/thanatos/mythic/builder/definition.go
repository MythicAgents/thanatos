// Main builder entrypoint. This is where the payload builder is defined and implemented
package builder

import (
	"path/filepath"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

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

var BuiltinCommands = []string{
	"sleep",
	"exit",
}

// Main entrypoint when Mythic executes the payload builder
func mythicBuildPayloadFunction(payloadBuildMsg agentstructs.PayloadBuildMessage) agentstructs.PayloadBuildResponse {
	return BuildPayload(MythicPayloadHandler{}, payloadBuildMsg)
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
