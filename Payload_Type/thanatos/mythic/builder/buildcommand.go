// Handles taking the parsed build parameters and converting it into the command used to
// build the payload
package builder

import (
	"fmt"
)

// Take the parsed build parameter command config and return a string containing the command to build
// the payload
func FormulateBuildCommand(configPath string, target string) string {
	cargoCommand := fmt.Sprintf("env CONFIG_BIN_PATH=%s cargo build --target %s --release", configPath, target)
	return cargoCommand
}
