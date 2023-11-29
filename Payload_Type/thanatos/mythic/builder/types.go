package builder

type PayloadBuildParameterArchitecture byte

const (
	PayloadBuildParameterArchitectureAmd64 PayloadBuildParameterArchitecture = iota
	PayloadBuildParameterArchitectureX86
)

func (arch PayloadBuildParameterArchitecture) String() string {
	switch arch {
	case PayloadBuildParameterArchitectureAmd64:
		return "amd64"
	case PayloadBuildParameterArchitectureX86:
		return "x86"
	}

	panic("Invalid architecture value")
}

func NewPayloadBuildParameterArchitecture(arch string) *PayloadBuildParameterArchitecture {
	switch arch {
	case "amd64":
		val := new(PayloadBuildParameterArchitecture)
		*val = PayloadBuildParameterArchitectureAmd64
		return val
	case "x86":
		val := new(PayloadBuildParameterArchitecture)
		*val = PayloadBuildParameterArchitectureX86
		return val
	}

	return nil
}

type PayloadBuildParameterInitOptions string

const (
	PayloadBuildParameterInitOptionNone        PayloadBuildParameterInitOptions = "none"
	PayloadBuildParameterInitOptionSpawnThread PayloadBuildParameterInitOptions = "Spawn Thread (Windows Only)"
	PayloadBuildParameterInitOptionDaemonize   PayloadBuildParameterInitOptions = "Daemonize (Linux Only)"
)

type PayloadBuildParameterCryptoLibrary string

const (
	PayloadBuildParameterCryptoLibrarySystem   PayloadBuildParameterCryptoLibrary = "system (wincrypto-ng/openssl)"
	PayloadBuildParameterCryptoLibraryInternal PayloadBuildParameterCryptoLibrary = "internal"
)

type PayloadBuildParameterStaticOption string

const (
	PayloadBuildParameterStaticOptionOpenSSL PayloadBuildParameterStaticOption = "openssl"
	PayloadBuildParameterStaticOptionLibCurl PayloadBuildParameterStaticOption = "libcurl"
)

type PayloadBuildParameterOutputFormat string

const (
	PayloadBuildParameterOutputFormatExecutable        PayloadBuildParameterOutputFormat = "executable"
	PayloadBuildParameterOutputFormatSharedLibrary     PayloadBuildParameterOutputFormat = "Shared Library (Run on load)"
	PayloadBuildParameterOutputFormatSharedLibraryInit PayloadBuildParameterOutputFormat = "Shared Library (.dll/.so with export name 'init')"
	PayloadBuildParameterOutputFormatWindowsShellcode  PayloadBuildParameterOutputFormat = "Windows Shellcode"
)
