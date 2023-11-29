package builder

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/uuid"
)

const buildTestDataDir string = "../testdata/buildtests"

type expectCompareOptions struct {
	Contains    *string `json:"contains"`
	Regex       *string `json:"regex"`
	Is          *string `json:"is"`
	Insensitive *bool   `json:"case_insensitive"`
}

type expectValues struct {
	Success  bool                  `json:"success"`
	Message  *expectCompareOptions `json:"message"`
	Stdout   *expectCompareOptions `json:"stdout"`
	Stderr   *expectCompareOptions `json:"stderr"`
	Filename *expectCompareOptions `json:"filename"`
}

type testSpec struct {
	Filename        string                               `json:"filename"`
	CommandList     []string                             `json:"commands"`
	SelectedOS      string                               `json:"selected_os"`
	BuildParameters map[string]interface{}               `json:"build_parameters"`
	C2Profiles      []agentstructs.PayloadBuildC2Profile `json:"c2profiles"`
	Expect          expectValues                         `json:"expect"`
}

type MockPayloadBuilder struct{}

func (builder MockPayloadBuilder) Build(command string) ([]byte, error) {
	return make([]byte, 0), nil
}

func TestPayloadMockBuild(t *testing.T) {
	testSpecs, err := os.ReadDir(buildTestDataDir)
	if err != nil {
		t.Fatal(err)
	}

	builder := MockPayloadBuilder{}

	for _, specPath := range testSpecs {
		t.Run(strings.TrimSuffix(specPath.Name(), ".json"), func(t *testing.T) {
			rawData, err := os.ReadFile(filepath.Join(buildTestDataDir, specPath.Name()))
			if err != nil {
				t.Fatal(err)
			}

			testData := testSpec{}
			if err := json.Unmarshal(rawData, &testData); err != nil {
				t.Fatal(err)
			}

			payloadUUID := uuid.NewString()
			payloadFileUUID := uuid.NewString()

			payloadBuildMsg := agentstructs.PayloadBuildMessage{
				PayloadType: "thanatos",
				Filename:    testData.Filename,
				CommandList: testData.CommandList,
				SelectedOS:  testData.SelectedOS,
				BuildParameters: agentstructs.PayloadBuildArguments{
					Parameters: testData.BuildParameters,
				},
				C2Profiles:         testData.C2Profiles,
				WrappedPayload:     nil,
				WrappedPayloadUUID: nil,
				PayloadUUID:        payloadUUID,
				PayloadFileUUID:    payloadFileUUID,
			}

			buildResult := buildPayload(payloadBuildMsg, builder)

			if buildResult.PayloadUUID != payloadUUID {
				t.Fatalf("Resulting payload UUID did not match expected UUID. Found '%s' expected '%s'", buildResult.PayloadUUID, payloadUUID)
			}

			if buildResult.Success != testData.Expect.Success {
				t.Logf("(buildResult.Success = %t) != (testData.Expect.Success = %t)", buildResult.Success, testData.Expect.Success)
				if buildResult.Success {
					t.Fatal("Payload build returned a successful status but expected a failed status")
				} else {
					t.Fatal("Payload build returned a failed status but expected a successful status")
				}
			}

			containsErrors := false

			if testData.Expect.Message != nil {
				compareData := testData.Expect.Message
				value := buildResult.BuildMessage

				if compareData.Contains != nil {
					if !strings.Contains(value, *compareData.Contains) {
						t.Logf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage)
						t.Logf("testData.Expect.Message.Contains = %s", *compareData.Contains)
						t.Log("Expected build message does not match returned build message")
						containsErrors = true
					}
				} else if compareData.Is != nil {
					if value != *compareData.Is {
						t.Logf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage)
						t.Logf("testData.Expect.Message.Is = %s", *compareData.Is)
						t.Log("Expected build message does not match returned build message")
						containsErrors = true
					}
				} else if compareData.Regex != nil {
					re := regexp.MustCompile(*compareData.Regex)
					if re.FindStringIndex(value) == nil {
						t.Logf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage)
						t.Logf("testData.Expect.Message.Regex = %s", *compareData.Regex)
						t.Log("Expected build message does not match returned build message")
						containsErrors = true
					}
				}
			}

			if testData.Expect.Stdout != nil {
				compareData := testData.Expect.Stdout
				value := buildResult.BuildStdOut

				if compareData.Contains != nil {
					if !strings.Contains(value, *compareData.Contains) {
						t.Logf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut)
						t.Logf("testData.Expect.Stdout.Contains = %s", *compareData.Contains)
						t.Log("Expected build stdout does not match returned build stdout")
						containsErrors = true
					}
				} else if compareData.Is != nil {
					if value != *compareData.Is {
						t.Logf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut)
						t.Logf("testData.Expect.StdOut.Is = %s", *compareData.Is)
						t.Log("Expected build stdout does not match returned build stdout")
						containsErrors = true
					}
				} else if compareData.Regex != nil {
					re := regexp.MustCompile(*compareData.Regex)
					if re.FindStringIndex(value) == nil {
						t.Logf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut)
						t.Logf("testData.Expect.StdOut.Regex = %s", *compareData.Regex)
						t.Log("Expected build stdout does not match returned build stdout")
						containsErrors = true
					}
				}
			}

			if testData.Expect.Stderr != nil {
				compareData := testData.Expect.Stderr
				value := buildResult.BuildStdErr

				if compareData.Contains != nil {
					if !strings.Contains(value, *compareData.Contains) {
						t.Logf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr)
						t.Logf("testData.Expect.Stderr.Contains = %s", *compareData.Contains)
						t.Log("Expected build stderr does not match returned build stderr")
						containsErrors = true
					}
				} else if compareData.Is != nil {
					if value != *compareData.Is {
						t.Logf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr)
						t.Logf("testData.Expect.Stderr.Is = %s", *compareData.Is)
						t.Log("Expected build stderr does not match returned build stderr")
						containsErrors = true
					}
				} else if compareData.Regex != nil {
					re := regexp.MustCompile(*compareData.Regex)
					if re.FindStringIndex(value) == nil {
						t.Logf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr)
						t.Logf("testData.Expect.Stderr.Regex = %s", *compareData.Regex)
						t.Log("Expected build stderr does not match returned build stderr")
						containsErrors = true
					}
				}
			}

			if testData.Expect.Filename != nil {
				if buildResult.UpdatedFilename != nil {
					compareData := testData.Expect.Filename
					value := *buildResult.UpdatedFilename

					if compareData.Contains != nil {
						if !strings.Contains(value, *compareData.Contains) {
							t.Logf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename)
							t.Logf("testData.Expect.Filename.Contains = %s", *compareData.Contains)
							t.Log("Expected updated filename does not match returned filename")
							containsErrors = true
						}
					} else if compareData.Is != nil {
						if value != *compareData.Is {
							t.Logf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename)
							t.Logf("testData.Expect.Filename.Is = %s", *compareData.Is)
							t.Log("Expected updated filename does not match returned filename")
							containsErrors = true
						}
					} else if compareData.Regex != nil {
						re := regexp.MustCompile(*compareData.Regex)
						if re.FindStringIndex(value) == nil {
							t.Logf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename)
							t.Logf("testData.Expect.Filename.Regex = %s", *compareData.Regex)
							t.Log("Expected updated filename does not match returned filename")
							containsErrors = true
						}
					}
				} else {
					logMsgFormatStr := "%s = %s"
					logMsg := ""
					if testData.Expect.Filename.Contains != nil {
						logMsg = fmt.Sprintf(logMsgFormatStr, "testData.Expect.Filename.Contains", testData.Expect.Filename.Contains)
					} else if testData.Expect.Filename.Is != nil {
						logMsg = fmt.Sprintf(logMsgFormatStr, "testData.Expect.Filename.Is", testData.Expect.Filename.Is)
					} else if testData.Expect.Filename.Regex != nil {
						logMsg = fmt.Sprintf(logMsgFormatStr, "testData.Expect.Filename.Regex", testData.Expect.Filename.Regex)
					}

					if testData.Expect.Filename.Insensitive != nil {
						logMsg = fmt.Sprintf("%s, case_insensitive = %t", logMsg, *testData.Expect.Filename.Insensitive)
					}

					t.Log("buildResult.UpdatedFilename = nil")
					t.Log(logMsg)
					t.Fatal("Build result did not return an updated filename but expected it to be present")
				}
			}

			if containsErrors {
				t.Fatalf("Test '%s' failed", t.Name())
			}
		})
	}
}
