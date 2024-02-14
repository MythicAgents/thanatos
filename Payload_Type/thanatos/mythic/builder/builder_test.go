// Tests building a payload
package builder

import (
	"testing"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

var testCases = []TestCase{
	{
		Name:       "linux_amd64_basic",
		Filename:   "thanatos",
		SelectedOS: agentstructs.SUPPORTED_OS_LINUX,
		CommandList: []string{
			"exit",
			"sleep",
		},
		BuildParameters: map[string]interface{}{
			"architecture":       string(PayloadBuildParameterArchitectureAmd64),
			"initoptions":        string(PayloadBuildParameterInitOptionNone),
			"connection_retries": 1,
			"cryptolib":          string(PayloadBuildParameterCryptoLibraryInternal),
			"working_hours":      "00:00-23:59",
			"output":             string(PayloadBuildParameterOutputFormatExecutable),
		},
		C2Profiles: []agentstructs.PayloadBuildC2Profile{
			{
				Name:  "http",
				IsP2P: false,
				Parameters: map[string]interface{}{
					"callback_port":            80,
					"killdate":                 "2099-01-01",
					"encrypted_exchange_check": true,
					"callback_jitter":          23,
					"headers": map[string]interface{}{
						"User-Agent": "test",
					},
					"AESPSK": map[string]string{
						"value": "none",
					},
					"callback_host":     "http://mythic",
					"get_uri":           "index",
					"post_uri":          "data",
					"query_path_name":   "q",
					"proxy_host":        "",
					"proxy_user":        "",
					"proxy_port":        "",
					"proxy_pass":        "",
					"callback_interval": 10,
				},
			},
		},
		Expect: TestCaseResult{
			Success:      true,
			BuildCommand: "cargo build -p thanatos_binary --target x86_64-unknown-linux-gnu --features crypto-internal,http --release",
		},
	},
	{
		Name:       "linux_amd64_system_crypto",
		Filename:   "thanatos",
		SelectedOS: agentstructs.SUPPORTED_OS_LINUX,
		CommandList: []string{
			"exit",
			"sleep",
		},
		BuildParameters: map[string]interface{}{
			"architecture":       string(PayloadBuildParameterArchitectureAmd64),
			"initoptions":        string(PayloadBuildParameterInitOptionNone),
			"connection_retries": 1,
			"cryptolib":          string(PayloadBuildParameterCryptoLibrarySystem),
			"working_hours":      "00:00-23:59",
			"output":             string(PayloadBuildParameterOutputFormatExecutable),
		},
		C2Profiles: []agentstructs.PayloadBuildC2Profile{
			{
				Name:  "http",
				IsP2P: false,
				Parameters: map[string]interface{}{
					"callback_port":            80,
					"killdate":                 "2099-01-01",
					"encrypted_exchange_check": true,
					"callback_jitter":          23,
					"headers": map[string]interface{}{
						"User-Agent": "test",
					},
					"AESPSK": map[string]string{
						"value": "none",
					},
					"callback_host":     "http://mythic",
					"get_uri":           "index",
					"post_uri":          "data",
					"query_path_name":   "q",
					"proxy_host":        "",
					"proxy_user":        "",
					"proxy_port":        "",
					"proxy_pass":        "",
					"callback_interval": 10,
				},
			},
		},
		Expect: TestCaseResult{
			Success:      true,
			BuildCommand: "cargo build -p thanatos_binary --target x86_64-unknown-linux-gnu --features crypto-system,http --release",
		},
	},
	{
		Name:       "linux_amd64_hostnames",
		Filename:   "thanatos",
		SelectedOS: agentstructs.SUPPORTED_OS_LINUX,
		CommandList: []string{
			"exit",
			"sleep",
		},
		BuildParameters: map[string]interface{}{
			"architecture":       string(PayloadBuildParameterArchitectureAmd64),
			"initoptions":        string(PayloadBuildParameterInitOptionNone),
			"connection_retries": 1,
			"cryptolib":          string(PayloadBuildParameterCryptoLibraryInternal),
			"working_hours":      "00:00-23:59",
			"output":             string(PayloadBuildParameterOutputFormatExecutable),
			"hostnames": []interface{}{
				"myhost",
			},
		},
		C2Profiles: []agentstructs.PayloadBuildC2Profile{
			{
				Name:  "http",
				IsP2P: false,
				Parameters: map[string]interface{}{
					"callback_port":            80,
					"killdate":                 "2099-01-01",
					"encrypted_exchange_check": true,
					"callback_jitter":          23,
					"headers": map[string]interface{}{
						"User-Agent": "test",
					},
					"AESPSK": map[string]string{
						"value": "none",
					},
					"callback_host":     "http://mythic",
					"get_uri":           "index",
					"post_uri":          "data",
					"query_path_name":   "q",
					"proxy_host":        "",
					"proxy_user":        "",
					"proxy_port":        "",
					"proxy_pass":        "",
					"callback_interval": 10,
				},
			},
		},
		Expect: TestCaseResult{
			Success:      true,
			BuildCommand: "cargo build -p thanatos_binary --target x86_64-unknown-linux-gnu --features crypto-internal,hostnamecheck,http --release",
		},
	},
}

// Test function which mocks all of the payload building and tests the build responses
func TestPayloadMockBuild(t *testing.T) {
	RunTestCases(t, testCases, MockBuildPayloadHandler{})
}

// Test function which will build the payload in the test
func TestPayloadFullBuild(t *testing.T) {
	RunTestCases(t, testCases, FullBuildPayloadHandler{})
}
