package builder

import (
	"strings"
	"testing"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/uuid"
)

func RunTestCases(t *testing.T, testCases []TestCase, handler BuildHandler) {
	for _, testCase := range testCases {
		t.Run(testCase.Name, func(t *testing.T) {
			input := createBuildInput(testCase)
			buildResult := BuildPayload(handler, input)

			t.Logf("[BUILD MESSAGE]: %s", buildResult.BuildMessage)
			t.Logf("[BUILD STDOUT]: %s", buildResult.BuildStdOut)
			t.Logf("[BUILD STDERR]: %s", buildResult.BuildStdErr)

			checkResults(t, buildResult, input, testCase)
		})
	}
}

func createBuildInput(testCase TestCase) agentstructs.PayloadBuildMessage {
	payloadUUID := uuid.NewString()
	payloadFileUUID := uuid.NewString()

	inputPayloadBuildMsg := agentstructs.PayloadBuildMessage{
		PayloadUUID:     payloadUUID,
		PayloadFileUUID: payloadFileUUID,

		PayloadType: "thanatos",
		Filename:    testCase.Filename,
		SelectedOS:  testCase.SelectedOS,
		CommandList: testCase.CommandList,
		BuildParameters: agentstructs.PayloadBuildArguments{
			Parameters: testCase.BuildParameters,
		},
		C2Profiles: testCase.C2Profiles,
	}

	return inputPayloadBuildMsg
}

func checkResults(t *testing.T, result agentstructs.PayloadBuildResponse, input agentstructs.PayloadBuildMessage, testCase TestCase) {
	if result.Success && !testCase.Expect.Success {
		t.Fatalf("test case returned a success but was expected to fail")
	}

	if !result.Success && testCase.Expect.Success {
		t.Fatalf("test case returned a failure but was expected to succeed")
	}

	if result.PayloadUUID != input.PayloadUUID {
		t.Logf("[input.PayloadUUID]: %s", input.PayloadUUID)
		t.Logf("[result.PayloadUUID]: %s", result.PayloadUUID)
		t.Fatalf("input payload UUID does not match resulting payload UUID")
	}

	if !strings.Contains(result.BuildMessage, testCase.Expect.BuildCommand) {
		t.Logf("[result.BuildMessage]:\n%s", result.BuildMessage)
		t.Logf("[testCase.Expect.BuildCommand]: %s", testCase.Expect.BuildCommand)
		t.Fatalf("expected build command does not match resulting build command")
	}
}
