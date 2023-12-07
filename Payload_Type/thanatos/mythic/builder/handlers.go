// Implementations for the handler routines when the payload is being built
package builder

import (
	"errors"
	"os/exec"
	"strings"
	builderrors "thanatos/builder/errors"

	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

// Type for the handler routines when being built by Mythic
type MythicPayloadHandler struct{}

// This will build the agent using the specified command string
func (handler MythicPayloadHandler) Build(command string) ([]byte, error) {
	return make([]byte, 0), nil
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
