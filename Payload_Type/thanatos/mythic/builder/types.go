package builder

<<<<<<< HEAD
import (
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

=======
>>>>>>> c89d869 (Stage rewrite files)
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
<<<<<<< HEAD

// Generic handler interface for managing payload builds and RPC execution
type BuildHandler interface {
	PayloadBuilder
	MythicRPCExecutor
}

// Interface handling various payload build routines
type PayloadBuilder interface {
	// Method which takes in the raw command for building the agent and returns the contents
	// of the built payload for Mythic
	Build(command string) ([]byte, error)

	// Method to install a required target
	InstallTarget(target string) error
}

// Interface for execution Mythic RPC routines
type MythicRPCExecutor interface {
	UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error)
}
=======
>>>>>>> c89d869 (Stage rewrite files)
