// Handles taking the parsed build parameters and converting it into the command used to
// build the payload
package builder

import (
	"fmt"
)

// Take the parsed build parameter command config and return a string containing the command to build
// the payload
func FormulateBuildCommand(configPath string, target string, payloadConfig ParsedPayloadParameters) string {

	profile := ""
	if payloadConfig.C2Profiles.HttpProfile != nil {
		profile = "http"
	} else {
		panic("Unimplemented build profile")
	}

	output := ""
	switch payloadConfig.PayloadBuildParameters.Output {
	case PayloadBuildParameterOutputFormatExecutable:
		output = "binary"
	default:
		panic("Unimplemented output")
	}

	buildPackage := fmt.Sprintf("thanatos_%s_%s", profile, output)
	cargoCommand := fmt.Sprintf("env CONFIG=%s cargo build -p %s --target %s --release", configPath, buildPackage, target)
	return cargoCommand
}
