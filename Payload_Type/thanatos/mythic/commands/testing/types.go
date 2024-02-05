package testing

import "github.com/MythicMeta/MythicContainer/mythicrpc"

type TestCase struct {
	Name               string
	InputCLI           string
	PreTasking         TestCaseExpectedValues
	ResponseProcessing TestCaseResponseProcessingValues
	AgentResponse      TestCaseAgentResponseValue
}

type TestCaseExpectedValues struct {
	ParseArgsResult     TestCaseResultValue
	CreateTaskingResult TestCaseResultValue
}

type TestCaseResultValue struct {
	ArgValues []TestCaseArgValue
	Success   bool
}

type TestCaseArgValue struct {
	Name  string
	Value interface{}
}

type TestCaseResponseProcessingValues struct {
	CompletionFunctionResult TestCaseCompletionFunctionResult
	RPCResults               TestCaseRPCValues
}

type TestCaseCompletionFunctionResult struct {
	Success bool
}

type TestCaseAgentResponseValue struct {
	Completed  bool
	UserOutput string
}

type TestCaseRPCValues struct {
	Response string
	Callback mythicrpc.MythicRPCCallbackCreateMessage
}

type MockRPCExecutor struct {
	Response string
	Callback mythicrpc.MythicRPCCallbackUpdateMessage
}
