package types

// Type specifiers for the build parameters. This is used for taking build options and
// turning them into clearly defined types. The purpose is to help identify build parameter
// option parsing issues at compile time instead of at runtime.

// Type for the target architecture of the payload
type PayloadBuildParameterArchitecture string

const (
	// Build payload is amd64 (64 bit)
	PayloadBuildParameterArchitectureAmd64 PayloadBuildParameterArchitecture = "amd64"

	// Build payload is x86 (32 bit)
	PayloadBuildParameterArchitectureX86 = "x86"
)

// Type for the Rust build mode
type PayloadBuildParameterRustBuildMode string

const (
	// Debug mode
	PayloadBuildParameterRustBuildModeDebug PayloadBuildParameterRustBuildMode = "debug"

	// Release Mode
	PayloadBuildParameterRustBuildModeRelease = "release"
)

// Type for the initial execution options
type PayloadBuildParameterInitAction string

const (
	// Payload should not modify the start routine
	PayloadBuildParameterInitActionNone PayloadBuildParameterInitAction = "none"

	// Payload should spawn a new thread when it is executed
	PayloadBuildParameterInitActionThread = "Spawn and run in a new thread"

	// Payload should fork and run in the background when it is executed
	PayloadBuildParameterInitActionFork = "Fork (Linux Only)"
)

// Type for the payload output format
type PayloadBuildParameterOutputFormat string

const (
	// Payload should be built into an executable
	PayloadBuildParameterOutputFormatExecutable PayloadBuildParameterOutputFormat = "executable"

	// Payload should be built into a shared library (DLL) which executes when it is loaded
	PayloadBuildParameterOutputFormatSharedLibraryInit PayloadBuildParameterOutputFormat = "Shared Library (run on load)"

	// Payload should be built with the entrypoint being an export named init
	PayloadBuildParameterOutputFormatSharedLibraryExport PayloadBuildParameterOutputFormat = "Shared Library (with export name)"

	// Payload should be built as shellcode for Windows
	PayloadBuildParameterOutputFormatWindowsShellcode PayloadBuildParameterOutputFormat = "Windows Shellcode"

	// Export the source code as a zip file
	PayloadBuildParameterOutputFormatSourceCode PayloadBuildParameterOutputFormat = "Source Code (zip)"
)
