// Implementations for the handler routines when the payload is being built
package builder

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"
	builderrors "thanatos/builder/errors"

	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

const AGENT_CODE_PATH = "agent"

// Type for the handler routines when being built by Mythic
type MythicPayloadHandler struct{}

// This will build the agent using the specified command string
func (handler MythicPayloadHandler) Build(target string, outform PayloadBuildParameterOutputFormat, command string) ([]byte, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return []byte{}, builderrors.Errorf("failed to get the current working directory: %s", err.Error())
	}

	agentCodePath := fmt.Sprintf("%s/%s", cwd, AGENT_CODE_PATH)

	cmd := exec.Command("/bin/bash", "-c", command)
	cmd.Dir = agentCodePath

	output, err := cmd.CombinedOutput()
	if err != nil {
		errorMsg := builderrors.Errorf("failed to build the agent: %s", err.Error())
		return []byte{}, errors.Join(builderrors.Errorf("output for command '/bin/bash -c %s:\n%s", command, string(output)), errorMsg)
	}

	outpath := fmt.Sprintf("%s/target/%s/release", agentCodePath, target)

	filename := ""
	if strings.Contains(target, "linux") {
		if outform == PayloadBuildParameterOutputFormatExecutable {
			filename = "thanatos"
		} else {
			filename = "libthanatos_core.so"
		}
	} else {
		if outform == PayloadBuildParameterOutputFormatExecutable {
			filename = "thanatos.exe"
		} else {
			filename = "thanatos_core.dll"
		}
	}

	payload, err := os.ReadFile(fmt.Sprintf("%s/%s", outpath, filename))
	if err != nil {
		return []byte{}, builderrors.Errorf("failed to open the built payload: %s", err.Error())
	}

	return payload, nil
}

// This will install a given Rust target if it does not exist
func (handler MythicPayloadHandler) InstallBuildTarget(target string) error {
	output, err := exec.Command("/bin/bash", "-c", "rustup target list").CombinedOutput()
	if err != nil {
		errorMsg := builderrors.Errorf("failed to list the currently installed Rust targets: %s", err.Error())
		return errors.Join(builderrors.Errorf("output for command '/bin/bash -c rustup target list':\n%s", string(output)), errorMsg)
	}

	for _, s := range strings.Split(string(output), "\n") {
		if strings.Contains(s, target+" ") {
			if strings.Contains(s, "(installed)") {
				return nil
			}
		}
	}

	command := []string{
		"/bin/bash",
		"-c",
		"rustup target add " + target,
	}

	output, err = exec.Command(command[0], command[1:]...).CombinedOutput()
	if err != nil {
		errorMsg := builderrors.Errorf("failed to install Rust target %s: %s", target, err.Error())
		return errors.Join(builderrors.Errorf("output for command '%s':\n%s", strings.Join(command, " "), string(output)), errorMsg)
	}

	return nil
}

// This updates the current build step in Mythic
func (handler MythicPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return mythicrpc.SendMythicRPCPayloadUpdateBuildStep(input)
}
