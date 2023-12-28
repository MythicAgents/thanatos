// Datatypes the builder uses
package builder

import (
	"github.com/MythicMeta/MythicContainer/mythicrpc"
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

/*
/// HTTP profile configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfigVars<'a> {
    callback_host: &'a str,
    callback_interval: usize,
    callback_jitter: u16,
    callback_port: u16,
    get_uri: &'a str,
    headers: HashMap<&'a str, &'a str>,
    killdate: usize,
    post_uri: &'a str,
    query_path_name: &'a str,
}
*/

/*
/// Payload configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigVars<'a> {
    uuid: Uuid,
    init_option: InitOption,
    working_hours_start: u64,
    working_hours_end: u64,
    connection_retries: usize,
    domains: Vec<[u8; 32]>,
    hostnames: Vec<[u8; 32]>,
    usernames: Vec<[u8; 32]>,
    tlsselfsigned: bool,
    spawn_to: &'a str,
    profile: HttpConfigVars<'a>,
}
*/

/*
UUID=a9fae67e-3006-440e-abae-57e836f961db
CONNECTION_RETRIES=1
WORKING_HOURS_START=0
WORKING_HOURS_END=86400
HTTP_CALLBACK_JITTER=23
HTTP_CALLBACK_HOST=http://mythic
HTTP_CALLBACK_PORT=80
HTTP_GET_URI=index
HTTP_CALLBACK_INTERVAL=1
HTTP_KILLDATE=4070908800
HTTP_HEADERS=eyJVc2VyLUFnZW50IjoidGVzdCJ9
HTTP_POST_URI=data
HTTP_QUERY_PATH_NAME=q
*/
