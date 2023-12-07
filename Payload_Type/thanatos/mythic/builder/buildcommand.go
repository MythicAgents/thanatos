// Handles taking the parsed build parameters and converting it into the command used to
// build the payload
package builder

import (
	"fmt"
	"log"
	"strings"
)

// Take the parsed build parameters and return a string containing the command to build
// the payload
func FormulateBuildCommand(parameters ParsedPayloadParameters, target string, uuid string) (string, error) {
	command := []string{"env"}

	envvars := map[string]string{}
	envvars["uuid"] = uuid
	envvars["connection_retries"] = fmt.Sprint(parameters.PayloadBuildParameters.ConnectionRetries)
	envvars["working_hours"] = 
	return "", nil
}
