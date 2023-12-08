// Datatypes the builder uses
package builder

import (
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

// Type for the payload architecture parameter
type PayloadBuildParameterArchitecture byte

const (
	// Build payload is amd64 (64 bit)
	PayloadBuildParameterArchitectureAmd64 PayloadBuildParameterArchitecture = iota

	// Build payload is x86 (32 bit)
	PayloadBuildParameterArchitectureX86
)

// Converts the architecture to a string
func (arch PayloadBuildParameterArchitecture) String() string {
	switch arch {
	case PayloadBuildParameterArchitectureAmd64:
		return "amd64"
	case PayloadBuildParameterArchitectureX86:
		return "x86"
	}

	panic("Invalid architecture value")
}

// Creates a new build parameter architecture from a specified value
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

// Type for the initial execution options
type PayloadBuildParameterInitOptions string

const (
	// Payload should not modify the start routine
	PayloadBuildParameterInitOptionNone PayloadBuildParameterInitOptions = "none"

	// Payload should spawn a new thread when it is executed
	PayloadBuildParameterInitOptionSpawnThread PayloadBuildParameterInitOptions = "Spawn Thread (Windows Only)"

	// Payload should fork and run in the background when it is executed
	PayloadBuildParameterInitOptionDaemonize PayloadBuildParameterInitOptions = "Daemonize (Linux Only)"
)

// Type for the specified crypto library
type PayloadBuildParameterCryptoLibrary string

const (
	// Payload should use the system's crypto library
	PayloadBuildParameterCryptoLibrarySystem PayloadBuildParameterCryptoLibrary = "system (wincrypto-ng/openssl)"

	// Payload should use the internal crypto functions
	PayloadBuildParameterCryptoLibraryInternal PayloadBuildParameterCryptoLibrary = "internal"
)

// Type for the static linking options
type PayloadBuildParameterStaticOption string

const (
	// Payload should statically link against openssl
	PayloadBuildParameterStaticOptionOpenSSL PayloadBuildParameterStaticOption = "openssl"

	// Payload should statically link against libcurl
	PayloadBuildParameterStaticOptionLibCurl PayloadBuildParameterStaticOption = "libcurl"
)

// Type for the payload output format
type PayloadBuildParameterOutputFormat string

const (
	// Payload should be built into an executable
	PayloadBuildParameterOutputFormatExecutable PayloadBuildParameterOutputFormat = "executable"

	// Payload should be built into a shared library (DLL) which executes when it is
	// loaded
	PayloadBuildParameterOutputFormatSharedLibrary PayloadBuildParameterOutputFormat = "Shared Library (Run on load)"

	// Payload should be built with the entrypoint being an export named init
	PayloadBuildParameterOutputFormatSharedLibraryInit PayloadBuildParameterOutputFormat = "Shared Library (.dll/.so with export name 'init')"

	// Payload should be built as shellcode for Windows
	PayloadBuildParameterOutputFormatWindowsShellcode PayloadBuildParameterOutputFormat = "Windows Shellcode"
)

// Generic handler interface for managing payload builds and RPC execution
type BuildHandler interface {
	PayloadBuilder
	MythicRPCExecutor
}

// Interface handling various payload build routines
type PayloadBuilder interface {
	// Method which takes in the raw command for building the agent and returns the contents
	// of the built payload for Mythic
	Build(target string, outform PayloadBuildParameterOutputFormat, command string) ([]byte, error)

	// Method to install a required target
	InstallBuildTarget(target string) error
}

// Interface for execution Mythic RPC routines
type MythicRPCExecutor interface {
	// Updates the build step in Mythic
	UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error)
}
