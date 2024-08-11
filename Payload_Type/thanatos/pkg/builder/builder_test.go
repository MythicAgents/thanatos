package builder

import (
	"testing"

	"github.com/MythicAgents/thanatos/proto/config"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/protobuf/testing/protocmp"
)

type configTestCase struct {
	buildParameters map[string]interface{}
	profiles        []agentstructs.PayloadBuildC2Profile
	expected        *config.Config
}

func TestCreateConfig(t *testing.T) {
	var createConfigTestCases = map[string]*configTestCase{
		"basic": {
			buildParameters: map[string]interface{}{
				"architecture":       "amd64",
				"buildmode":          "release",
				"initaction":         "none",
				"connection_retries": 10,
				"working_hours":      "00:00-23:59",
				"domains":            []interface{}{},
				"hostnames":          []interface{}{},
				"usernames":          []interface{}{},
				"tlsuntrusted":       false,
				"spawnto":            "",
				"libexport":          "",
				"output":             "",
			},
			profiles: []agentstructs.PayloadBuildC2Profile{
				{
					Name:  "http",
					IsP2P: false,
					Parameters: map[string]interface{}{
						"callback_port":            80,
						"killdate":                 "2099-01-01",
						"encrypted_exchange_check": true,
						"callback_jitter":          20,
						"headers": map[string]interface{}{
							"User-Agent": "user-agent",
						},
						"AESPSK": agentstructs.CryptoArg{
							Value: "none",
						},
						"callback_host":     "http://test",
						"get_uri":           "/",
						"post_uri":          "/post",
						"query_path_name":   "q",
						"proxy_host":        "",
						"proxy_port":        "",
						"proxy_user":        "",
						"proxy_pass":        "",
						"callback_interval": 10,
					},
				},
			},
			expected: &config.Config{
				Initaction:        config.InitAction_NONE,
				ConnectionRetries: 10,
				WorkingHours: &config.WorkingHours{
					Start:       0,
					End:         23*60 + 59,
					UseSystemTz: true,
					UtcOffset:   0,
				},
				SpawnTo:      "",
				Tlsuntrusted: false,
				Profile: &config.Config_Http{
					Http: &config.HttpConfig{
						CallbackPort:   80,
						Killdate:       4070908800,
						CallbackJitter: 20,
						Eke:            true,
						Headers: map[string]string{
							"User-Agent": "user-agent",
						},
						CallbackHost:     "http://test",
						GetUri:           "/",
						PostUri:          "/post",
						QueryPathName:    "q",
						Proxy:            nil,
						CallbackInterval: 10,
					},
				},
			},
		},
	}

	for name, testCase := range createConfigTestCases {
		t.Run(name, func(t *testing.T) {
			var payloadUuid = uuid.New()
			testCase.expected.Uuid = payloadUuid[:]

			buildMessage := agentstructs.PayloadBuildMessage{
				PayloadType: "thanatos",
				Filename:    "thanatos",
				CommandList: []string{},
				BuildParameters: agentstructs.PayloadBuildArguments{
					Parameters: testCase.buildParameters,
				},
				C2Profiles:  testCase.profiles,
				PayloadUUID: payloadUuid.String(),
			}

			result, err := CreateConfig(buildMessage)
			if err != nil {
				t.Fatal(err)
			}

			if d := cmp.Diff(testCase.expected, result, protocmp.Transform()); d != "" {
				t.Errorf("test case failed: %s", d)
			}
		})
	}
}
