// Implementations for the handler routines when the payload is being built
package builder

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/MythicAgents/thanatos/builder/types"
	thanatoserror "github.com/MythicAgents/thanatos/errors"

	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

const AGENT_CODE_PATH = "../agent"

// Type for the handler routines when being built by Mythic
type MythicPayloadHandler struct{}

// This will build the agent using the specified command string
func (handler MythicPayloadHandler) Build(command string, target string, output types.PayloadBuildParameterOutputFormat) ([]byte, error) {
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

// This updates the current build step in Mythic
func (handler MythicPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return mythicrpc.SendMythicRPCPayloadUpdateBuildStep(input)
}
