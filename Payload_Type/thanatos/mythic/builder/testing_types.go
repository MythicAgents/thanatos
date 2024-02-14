package builder

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

type TestCase struct {
	Name            string
	Filename        string
	SelectedOS      string
	CommandList     []string
	BuildParameters map[string]interface{}
	C2Profiles      []agentstructs.PayloadBuildC2Profile

	Expect TestCaseResult
}

type TestCaseResult struct {
	Success      bool
	BuildCommand string
}

// Type which contains the mock implementations of the handler routines. This will
// essentially "no-op" expensive function or Mythic RPC calls
type MockBuildPayloadHandler struct{}

// Mock implementation for the payload build
func (handler MockBuildPayloadHandler) Build(target string, config ParsedPayloadParameters, command string) ([]byte, error) {
	return []byte{}, nil
}

// Mock implementation for updating a build step in Mythic
func (handler MockBuildPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
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
func (handler FullBuildPayloadHandler) Build(target string, config ParsedPayloadParameters, command string) ([]byte, error) {
	return MythicPayloadHandler{}.Build(target, config, command)
}

// Runs the mock Mythic RPC function
func (handler FullBuildPayloadHandler) UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error) {
	return MockBuildPayloadHandler{}.UpdateBuildStep(input)
}
