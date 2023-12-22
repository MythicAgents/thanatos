// Handles taking the parsed build parameters and converting it into the command used to
// build the payload
package builder

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"sort"
	"strings"
	builderrors "thanatos/builder/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

type BuildCommandConfig struct {
	EnvVars  []map[string]string
	Features []string
}

func (c *BuildCommandConfig) String() string {
	output := ""
	for _, m := range c.EnvVars {
		for key, val := range m {
			output += fmt.Sprintf("%s=%s\n", key, val)
		}
	}

	return output
}

// Create a mapping for the build parameters
func CreateCommandConfig(parameters ParsedPayloadParameters, os string, uuid string) (BuildCommandConfig, error) {
	config := BuildCommandConfig{}

	payloadvars := map[string]string{}
	featureFlags := []string{}

	// uuid
	payloadvars["uuid"] = uuid

	// Connection retries
	payloadvars["connection_retries"] = fmt.Sprint(parameters.PayloadBuildParameters.ConnectionRetries)

	// Working hours
	payloadvars["working_hours_start"] = fmt.Sprint(int(parameters.PayloadBuildParameters.WorkingHours.StartTime.Seconds()))
	payloadvars["working_hours_end"] = fmt.Sprint(int(parameters.PayloadBuildParameters.WorkingHours.EndTime.Seconds()))

	// Init options
	switch parameters.PayloadBuildParameters.InitOptions {
	case PayloadBuildParameterInitOptionSpawnThread:
		payloadvars["init_option"] = "thread"
	case PayloadBuildParameterInitOptionDaemonize:
		payloadvars["init_option"] = "daemonize"
	}

	// TODO: Crypto library
	switch parameters.PayloadBuildParameters.CryptoLib {
	case PayloadBuildParameterCryptoLibraryInternal:
		featureFlags = append(featureFlags, "cryptolib?/internal")
	case PayloadBuildParameterCryptoLibrarySystem:
		featureFlags = append(featureFlags, "cryptolib?/system")
	}

	// Domain list
	if len(parameters.PayloadBuildParameters.DomainList) > 0 {
		featureFlags = append(featureFlags, "domaincheck")
	}

	// Hostname list
	if len(parameters.PayloadBuildParameters.HostnameList) > 0 {
		featureFlags = append(featureFlags, "hostnamecheck")
	}

	// Username list
	if len(parameters.PayloadBuildParameters.UsernameList) > 0 {
		featureFlags = append(featureFlags, "usernamecheck")
	}

	// Domain list
	if len(parameters.PayloadBuildParameters.DomainList) > 0 {
		payloadvars["domain_list"] = strings.Join(parameters.PayloadBuildParameters.DomainList, ",")
		featureFlags = append(featureFlags, "domaincheck")
	}

	// Hostname list
	if len(parameters.PayloadBuildParameters.HostnameList) > 0 {
		payloadvars["hostname_list"] = strings.Join(parameters.PayloadBuildParameters.HostnameList, ",")
		featureFlags = append(featureFlags, "hostnamecheck")
	}

	// Username list
	if len(parameters.PayloadBuildParameters.UsernameList) > 0 {
		payloadvars["username_list"] = strings.Join(parameters.PayloadBuildParameters.UsernameList, ",")
		featureFlags = append(featureFlags, "usernamecheck")
	}

	// Static options
	if os == agentstructs.SUPPORTED_OS_LINUX {
		for _, option := range parameters.PayloadBuildParameters.StaticOptions {
			switch option {
			case PayloadBuildParameterStaticOptionOpenSSL:
				payloadvars["OPENSSL_STATIC"] = "yes"
			case PayloadBuildParameterStaticOptionLibCurl:
				if parameters.C2Profiles.HttpProfile != nil {
					featureFlags = append(featureFlags, "curl/static-curl")
				}
			}
		}
	}

	// TLS self signed
	if parameters.C2Profiles.HttpProfile != nil {
		featureFlags = append(featureFlags, "http-profile/tlsselfsigned")
	}

	// Spawn to
	if len(parameters.PayloadBuildParameters.SpawnTo) > 0 {
		payloadvars["spawn_to"] = parameters.PayloadBuildParameters.SpawnTo
	}

	envVars := map[string]string{}
	for key, val := range payloadvars {
		newkey := strings.ToUpper(key)
		envVars[newkey] = val
	}
	config.EnvVars = append(config.EnvVars, envVars)

	// HTTP C2 profile
	if parameters.C2Profiles.HttpProfile != nil {
		profile := *parameters.C2Profiles.HttpProfile

		profilevars := map[string]string{}

		profilevars["callback_port"] = fmt.Sprint(profile.CallbackPort)
		profilevars["killdate"] = fmt.Sprint(profile.Killdate.Unix())

		if profile.EncryptedExchangeCheck {
			featureFlags = append(featureFlags, "http-profile/eke")
		}

		profilevars["callback_jitter"] = fmt.Sprint(profile.CallbackJitter)

		headers_json, err := json.Marshal(profile.Headers)
		if err != nil {
			return BuildCommandConfig{}, builderrors.Errorf("failed to marshal HTTP profile headers: %s", err.Error())
		}

		profilevars["headers"] = base64.StdEncoding.EncodeToString(headers_json)

		if profile.CryptoInfo != nil {
			profilevars["encryption_key"] = profile.CryptoInfo.Key
		}

		profilevars["callback_host"] = profile.CallbackHost
		profilevars["get_uri"] = profile.GetUri
		profilevars["post_uri"] = profile.PostUri
		profilevars["query_path_name"] = profile.QueryPathName

		if profile.ProxyInfo != nil {
			proxyinfo_json, err := json.Marshal(*profile.ProxyInfo)
			if err != nil {
				return BuildCommandConfig{}, builderrors.Errorf("failed to marshal HTTP proxy info: %s", err.Error())
			}

			profilevars["proxy_info"] = base64.StdEncoding.EncodeToString(proxyinfo_json)
		}

		profilevars["callback_interval"] = fmt.Sprint(profile.CallbackInterval)

		envVars = map[string]string{}
		for key, val := range profilevars {
			newkey := fmt.Sprintf("HTTP_%s", strings.ToUpper(key))
			envVars[newkey] = val
		}

		config.EnvVars = append(config.EnvVars, envVars)
	}

	sort.Strings(featureFlags)
	config.Features = featureFlags
	return config, nil
}

// Take the parsed build parameter command config and return a string containing the command to build
// the payload
func FormulateBuildCommand(config BuildCommandConfig, target string) (string, error) {
	buildCommand := []string{"env"}

	for _, m := range config.EnvVars {
		for key, val := range m {
			buildCommand = append(buildCommand, fmt.Sprintf("%s=%s", key, val))
		}
	}

	cargoCommand := fmt.Sprintf("cargo build --target %s --release", target)

	buildCommand = append(buildCommand, strings.Split(cargoCommand, " ")...)
	return strings.Join(buildCommand, " "), nil
}
