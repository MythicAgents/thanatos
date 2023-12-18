// Tests building a payload
package builder

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"regexp"
	"strings"
	"testing"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
	"github.com/google/uuid"
)

// Directory which holds the JSON files with the test data. This data includes the
// build/profile parameters and expected results from the build
const buildTestDataDir string = "./testdata/buildtests"

var (
	savedCwd string = ""
)

// Options for when testing if the build succeeded. This is for checking if the result
// BuildStdout, BuildStderr and BuildMessage contain correct values
type expectCompareOptions struct {
	// Option to check if the build result contains a specified string
	Contains *string `json:"contains"`

	// Option to check if the build result matches a regex pattern
	Regex *string `json:"regex"`

	// Option to check if the build result matches a specified string exactly
	Is *string `json:"is"`

	// Modifier signifying that the comparison should be case insensitive
	Insensitive *bool `json:"case_insensitive"`
}

// Expected BuildStdout, BuildStderr and BuildMessage values from a payload build
type expectValues struct {
	// Whether the build should be successful or not
	Success bool `json:"success"`

	// Expected build message
	Message *expectCompareOptions `json:"message"`

	// Expected build stdout
	Stdout *expectCompareOptions `json:"stdout"`

	// Expected build stderr
	Stderr *expectCompareOptions `json:"stderr"`

	// Expected new filename
	Filename *expectCompareOptions `json:"filename"`
}

// Definition for a new test. This contains the build parameters for the build along with
// a set of expected results
type testSpec struct {
	// Input filename for the build
	Filename string `json:"filename"`

	// List of commands for the build
	CommandList []string `json:"commands"`

	// Selected OS for the build
	SelectedOS string `json:"selected_os"`

	// Payload parameters for the build
	BuildParameters map[string]interface{} `json:"build_parameters"`

	// C2 profile parameters for the build
	C2Profiles []agentstructs.PayloadBuildC2Profile `json:"c2profiles"`

	// Expected build results
	Expect expectValues `json:"expect"`
}

// Type which contains the mock implementations of the handler routines. This will
// essentially "no-op" expensive function or Mythic RPC calls
type MockPayloadHandler struct{}

// Mock implementation for the payload build
func (handler MockPayloadHandler) Build(target string, outform PayloadBuildParameterOutputFormat, command string) ([]byte, error) {
	return []byte{}, nil
}

// Mock implementation for installing a Rust target
func (handler MockPayloadHandler) InstallBuildTarget(target string) error {
	return nil
}

// Mock implementation for updating a build step in Mythic
func (handler MockPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	response := mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse{
		Success: true,
		Error:   "",
	}
	return &response, nil
}

// Type which contains the full implementations for building a payload. This will build
// the payload and install the required Rust tool chain. This will mock the Mythic RPC
// calls
type FullBuildPayloadHandler struct{}

// Runs the real build command for the build handler
func (handler FullBuildPayloadHandler) Build(target string, outform PayloadBuildParameterOutputFormat, command string) ([]byte, error) {
	return MythicPayloadHandler{}.Build(target, outform, command)
}

// Runs the real Rust target install command for the build handler
func (handler FullBuildPayloadHandler) InstallBuildTarget(target string) error {
	return MythicPayloadHandler{}.InstallBuildTarget(target)
}

// Runs the mock Mythic RPC function
func (handler FullBuildPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return MockPayloadHandler{}.UpdateBuildStep(input)
}

// Prints out a set of data using the testing logger
func testLogPrintData(t *testing.T, data ...any) {
	for _, v := range data {
		p, err := json.MarshalIndent(v, "", "  ")
		if err != nil {
			continue
		}

		typeName := reflect.TypeOf(v).String()
		t.Logf("%s:\n%s", typeName, string(p))
	}
}

// Checks the payload build results with the expected results
func checkResults(t *testing.T, payloadUUID string, buildResult agentstructs.PayloadBuildResponse, testData testSpec) {
	if buildResult.PayloadUUID != payloadUUID {
		testLogPrintData(t, testData, buildResult)
		t.Fatalf("Resulting payload UUID did not match expected UUID. Found '%s' expected '%s'", buildResult.PayloadUUID, payloadUUID)
	}

	if buildResult.Success != testData.Expect.Success {
		testLogPrintData(t, testData, buildResult)
		t.Logf("(buildResult.Success = %t) != (testData.Expect.Success = %t)", buildResult.Success, testData.Expect.Success)
		if buildResult.Success {
			t.Fatal("Payload build was successful but the test expected it to fail")
		} else {
			t.Fatal("Payload build was unsuccessful but the test expected it to succeed")
		}
	}

	logMsgBuffer := []string{}

	if testData.Expect.Message != nil {
		compareData := testData.Expect.Message
		value := buildResult.BuildMessage

		if compareData.Contains != nil {
			if !strings.Contains(value, *compareData.Contains) {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expec.Message.Contains = %s", *compareData.Contains))
				logMsgBuffer = append(logMsgBuffer, "Expected build message does not match returned build message")
			}
		} else if compareData.Is != nil {
			if value != *compareData.Is {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Message.Is = %s", *compareData.Is))
				logMsgBuffer = append(logMsgBuffer, "Expected build message does not match returned build message")
			}
		} else if compareData.Regex != nil {
			re := regexp.MustCompile(*compareData.Regex)
			if re.FindStringIndex(value) == nil {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildMessage:\n%s\n", buildResult.BuildMessage))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Message.Regex = %s", *compareData.Regex))
				logMsgBuffer = append(logMsgBuffer, "Expected build message does not match returned build message")
			}
		}
	}

	if testData.Expect.Stdout != nil {
		compareData := testData.Expect.Stdout
		value := buildResult.BuildStdOut

		if compareData.Contains != nil {
			if !strings.Contains(value, *compareData.Contains) {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Stdout.Contains = %s", *compareData.Contains))
				logMsgBuffer = append(logMsgBuffer, "Expected build stdout does not match returned build stdout")
			}
		} else if compareData.Is != nil {
			if value != *compareData.Is {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.StdOut.Is = %s", *compareData.Is))
				logMsgBuffer = append(logMsgBuffer, "Expected build stdout does not match returned build stdout")
			}
		} else if compareData.Regex != nil {
			re := regexp.MustCompile(*compareData.Regex)
			if re.FindStringIndex(value) == nil {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdOut:\n%s\n", buildResult.BuildStdOut))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.StdOut.Regex = %s", *compareData.Regex))
				logMsgBuffer = append(logMsgBuffer, "Expected build stdout does not match returned build stdout")
			}
		}
	}

	if testData.Expect.Stderr != nil {
		compareData := testData.Expect.Stderr
		value := buildResult.BuildStdErr

		if compareData.Contains != nil {
			if !strings.Contains(value, *compareData.Contains) {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Stderr.Contains = %s", *compareData.Contains))
				logMsgBuffer = append(logMsgBuffer, "Expected build stderr does not match returned build stderr")
			}
		} else if compareData.Is != nil {
			if value != *compareData.Is {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Stderr.Is = %s", *compareData.Is))
				logMsgBuffer = append(logMsgBuffer, "Expected build stderr does not match returned build stderr")
			}
		} else if compareData.Regex != nil {
			re := regexp.MustCompile(*compareData.Regex)
			if re.FindStringIndex(value) == nil {
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.BuildStdErr:\n%s\n", buildResult.BuildStdErr))
				logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Stderr.Regex = %s", *compareData.Regex))
				logMsgBuffer = append(logMsgBuffer, "Expected build stderr does not match returned build stderr")
			}
		}
	}

	if testData.Expect.Filename != nil {
		if buildResult.UpdatedFilename != nil {
			compareData := testData.Expect.Filename
			value := *buildResult.UpdatedFilename

			if compareData.Contains != nil {
				if !strings.Contains(value, *compareData.Contains) {
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename))
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Filename.Contains = %s", *compareData.Contains))
					logMsgBuffer = append(logMsgBuffer, "Expected updated filename does not match returned filename")
				}
			} else if compareData.Is != nil {
				if value != *compareData.Is {
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename))
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Filename.Is = %s", *compareData.Is))
					logMsgBuffer = append(logMsgBuffer, "Expected updated filename does not match returned filename")
				}
			} else if compareData.Regex != nil {
				re := regexp.MustCompile(*compareData.Regex)
				if re.FindStringIndex(value) == nil {
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("buildResult.UpdatedFilename = %s", *buildResult.UpdatedFilename))
					logMsgBuffer = append(logMsgBuffer, fmt.Sprintf("testData.Expect.Filename.Regex = %s", *compareData.Regex))
					logMsgBuffer = append(logMsgBuffer, "Expected updated filename does not match returned filename")
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

			logMsgBuffer = append(logMsgBuffer, "buildResult.UpdatedFilename = nil")
			logMsgBuffer = append(logMsgBuffer, logMsg)
			logMsgBuffer = append(logMsgBuffer, "Build result did not return an updated filename but expected it to be present")
		}
	}

	if len(logMsgBuffer) > 0 {
		testLogPrintData(t, testData, buildResult)
		for _, m := range logMsgBuffer {
			t.Log(m)
		}

		t.Logf("Test '%s' failed", t.Name())
		t.Fail()
	}
}

// Function which runs all of the tests with a specified handler
func testPayloadBuildImpl(t *testing.T, handler BuildHandler) {
	if savedCwd == "" {
		savedCwd, _ = os.Getwd()
	}

	os.Chdir(savedCwd + "/../..")

	cwd, _ := os.Getwd()
	t.Log(cwd)

	testSpecs, err := os.ReadDir(filepath.Join("mythic", buildTestDataDir))
	if err != nil {
		t.Fatal(err)
	}

	for _, specPath := range testSpecs {
		testDataSpecPath := specPath.Name()

		t.Run(strings.TrimSuffix(specPath.Name(), ".json"), func(t *testing.T) {
			t.Parallel()

			rawData, err := os.ReadFile(filepath.Join("mythic", buildTestDataDir, testDataSpecPath))
			if err != nil {
				t.Fatal(err)
			}

			testData := testSpec{}
			if err := json.Unmarshal(rawData, &testData); err != nil {
				t.Fatal(err)
			}

			testLogPrintData(t, testData)

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

			buildResult := buildPayload(payloadBuildMsg, handler)
			t.Logf("[BUILD MESSAGE]: %s", buildResult.BuildMessage)
			t.Logf("[BUILD STDOUT]: %s", buildResult.BuildStdOut)
			t.Logf("[BUILD STDERR]: %s", buildResult.BuildStdErr)
			checkResults(t, payloadUUID, buildResult, testData)
		})
	}
}

// Test function which mocks all of the payload building and Mythic RPC functions
// This test can be skipped by setting the `BUILDTEST` environment variable to "fullonly"
func TestPayloadMockBuild(t *testing.T) {
	handler := MockPayloadHandler{}
	testPayloadBuildImpl(t, handler)
}

// Test function which will build the payload in the test
// This test will not run unless the `BUILDTEST` environment variable is set to "full"
func TestPayloadFullBuild(t *testing.T) {
	handler := FullBuildPayloadHandler{}
	testPayloadBuildImpl(t, handler)
}
