// Datatypes the builder uses
package builder

import (
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

type SupportedCommand string

const (
	ExitCommand  SupportedCommand = "exit"
	SleepCommand SupportedCommand = "sleep"
)

// Type for the payload architecture parameter
type PayloadBuildParameterArchitecture string

const (
	// Build payload is amd64 (64 bit)
	PayloadBuildParameterArchitectureAmd64 PayloadBuildParameterArchitecture = "amd64"

	// Build payload is x86 (32 bit)
	PayloadBuildParameterArchitectureX86 PayloadBuildParameterArchitecture = "x86"
)

// Type for the initial execution options
type PayloadBuildParameterInitOptions string

const (
	// Payload should not modify the start routine
	PayloadBuildParameterInitOptionNone PayloadBuildParameterInitOptions = "none"

	// Payload should spawn a new thread when it is executed
	PayloadBuildParameterInitOptionSpawnThread PayloadBuildParameterInitOptions = "Spawn New Thread"

	// Payload should fork and run in the background when it is executed
	PayloadBuildParameterInitOptionFork PayloadBuildParameterInitOptions = "Fork On Run (Linux Only)"
)

// Type for the specified crypto library
type PayloadBuildParameterCryptoLibrary string

const (
	// Payload should use the system's crypto library
	PayloadBuildParameterCryptoLibrarySystem PayloadBuildParameterCryptoLibrary = "system (Windows CNG/Linux OpenSSL)"

	// Payload should use the internal crypto functions
	PayloadBuildParameterCryptoLibraryInternal PayloadBuildParameterCryptoLibrary = "internal (RustCrypto)"
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

	// Payload should be built into a shared library (DLL) which executes when it is loaded
	PayloadBuildParameterOutputFormatSharedLibraryInit PayloadBuildParameterOutputFormat = "Shared Library (run on load)"

	// Payload should be built with the entrypoint being an export named init
	PayloadBuildParameterOutputFormatSharedLibraryExport PayloadBuildParameterOutputFormat = "Shared Library (with export)"

	// Payload should be built as a reflective library with an export to the reflective loader
	PayloadBuildParameterOutputFormatReflectiveSharedLibrary PayloadBuildParameterOutputFormat = "Reflective Shared Library (with export)"

	// Payload should be built as shellcode for Windows
	PayloadBuildParameterOutputFormatWindowsShellcode PayloadBuildParameterOutputFormat = "Windows Shellcode"

	// Export the source code as a zip file
	PayloadBuildParameterOutputFormatSourceCode PayloadBuildParameterOutputFormat = "Source Code (zip)"
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
	Build(target string, config ParsedPayloadParameters, command string) ([]byte, error)
}

// Interface for execution Mythic RPC routines
type MythicRPCExecutor interface {
	// Updates the build step in Mythic
	UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error)
}
