package builder

import (
	"errors"
	"fmt"
	"os"

	"github.com/MythicAgents/thanatos/pkg/builder/types"
	thanatoserror "github.com/MythicAgents/thanatos/pkg/errors"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

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
	payload, err := handler.Build(buildCommand, target, types.PayloadBuildParameterOutputFormat(output))
	if err != nil {
		payloadBuildResponse.BuildStdErr = errors.Join(thanatoserror.New("failed to build the payload"), err).Error()
		return payloadBuildResponse
	}

	payloadBuildResponse.Payload = &payload

	payloadBuildResponse.Success = true
	return payloadBuildResponse
}
