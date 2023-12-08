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
)

// Take the parsed build parameters and return a string containing the command to build
// the payload
func FormulateBuildCommand(parameters ParsedPayloadParameters, target string, uuid string) (string, error) {
	buildCommand := []string{"env"}

	featureFlags := []string{}
	payloadvars := map[string]string{}

	// Init options
	switch parameters.PayloadBuildParameters.InitOptions {
	case PayloadBuildParameterInitOptionSpawnThread:
		featureFlags = append(featureFlags, "init-thread")
	case PayloadBuildParameterInitOptionDaemonize:
		featureFlags = append(featureFlags, "init-daemonize")
	}

	// uuid
	payloadvars["uuid"] = uuid

	// Connection retries
	payloadvars["connection_retries"] = fmt.Sprint(parameters.PayloadBuildParameters.ConnectionRetries)

	// Crypto library
	switch parameters.PayloadBuildParameters.CryptoLib {
	case PayloadBuildParameterCryptoLibraryInternal:
		featureFlags = append(featureFlags, "cryptolib?/internal")
	case PayloadBuildParameterCryptoLibrarySystem:
		featureFlags = append(featureFlags, "cryptolib?/system")
	}

	// Working hours
	payloadvars["working_hours_start"] = fmt.Sprint(int(parameters.PayloadBuildParameters.WorkingHours.StartTime.Seconds()))
	payloadvars["working_hours_end"] = fmt.Sprint(int(parameters.PayloadBuildParameters.WorkingHours.EndTime.Seconds()))

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
	if strings.Contains(target, "linux") {
		for _, option := range parameters.PayloadBuildParameters.StaticOptions {
			switch option {
			case PayloadBuildParameterStaticOptionOpenSSL:
				buildCommand = append(buildCommand, "OPENSSL_STATIC=yes")
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

	payloadkeys := make([]string, 0, len(payloadvars))
	for key := range payloadvars {
		payloadkeys = append(payloadkeys, key)
	}
	sort.Strings(payloadkeys)

	for _, key := range payloadkeys {
		buildCommand = append(buildCommand, fmt.Sprintf("%s=%s", strings.ToUpper(key), payloadvars[key]))
	}

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
			return "", builderrors.Errorf("failed to marshal HTTP profile headers: %s", err.Error())
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
				return "", builderrors.Errorf("failed to marshal HTTP proxy info: %s", err.Error())
			}

			profilevars["proxy_info"] = base64.StdEncoding.EncodeToString(proxyinfo_json)
		}

		profilevars["callback_interval"] = fmt.Sprint(profile.CallbackInterval)

		profilekeys := make([]string, 0, len(profilevars))
		for key := range profilevars {
			profilekeys = append(profilekeys, key)
		}
		sort.Strings(profilekeys)

		for _, key := range profilekeys {
			buildCommand = append(buildCommand, fmt.Sprintf("HTTP_%s=%s", strings.ToUpper(key), profilevars[key]))
		}

	}

	cargoCommand := fmt.Sprintf("cargo build --target %s --release", target)
	_ = featureFlags
	//if len(featureFlags) > 0 {
	//	cargoCommand = fmt.Sprintf("%s --features %s", cargoCommand, strings.Join(featureFlags, ","))
	//}

	buildCommand = append(buildCommand, strings.Split(cargoCommand, " ")...)
	return strings.Join(buildCommand, " "), nil
}
