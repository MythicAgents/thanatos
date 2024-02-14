package builder

import (
	"fmt"
	"strings"
)

func FormulateBuildCommand(target string, configPath string, config ParsedPayloadParameters) string {
	output := ""
	switch config.BuildParameters.Output {
	case PayloadBuildParameterOutputFormatExecutable:
		output = "binary"
	default:
		panic("Unimplemented output")
	}

	buildPackage := fmt.Sprintf("thanatos_%s", output)

	features := []string{}
	features = append(features, config.Commands...)

	if config.BuildParameters.CryptoLib == PayloadBuildParameterCryptoLibrarySystem {
		features = append(features, "crypto-system")
	} else {
		features = append(features, "crypto-internal")
	}

	switch config.BuildParameters.InitOptions {
	case PayloadBuildParameterInitOptionFork:
		features = append(features, "init-fork")
	case PayloadBuildParameterInitOptionSpawnThread:
		features = append(features, "init-thread")
	}

	if len(config.BuildParameters.DomainList) > 0 {
		features = append(features, "domaincheck")
	}

	if len(config.BuildParameters.HostnameList) > 0 {
		features = append(features, "hostnamecheck")
	}

	if len(config.BuildParameters.UsernameList) > 0 {
		features = append(features, "usernamecheck")
	}

	if config.BuildParameters.TlsUntrusted {
		features = append(features, "tlsuntrusted")
	}

	if config.C2Profiles.HttpC2Profile != nil {
		features = append(features, "http")
	}

	if config.C2Profiles.TcpC2Profile != nil {
		features = append(features, "tcp")
	}

	featureString := strings.Join(features, ",")

	cargoCommand := fmt.Sprintf(
		"env CONFIG=%s cargo build -p %s --target %s --features %s --release",
		configPath,
		buildPackage,
		target,
		featureString,
	)

	return cargoCommand
}
