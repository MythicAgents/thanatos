package exit

import (
	"testing"
	cmdtesting "thanatos/commands/testing"
)

var testCases = []cmdtesting.TestCase{
	{
		Name:     "invalid_garbage",
		InputCLI: "foo",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},

	// Garbage value with spaces
	{
		Name:     "invalid_garbage_spaces",
		InputCLI: "this is a test",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},

	// Correct value (exit process)
	{
		Name:     "valid_process",
		InputCLI: "process",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "option",
						Value: string(exitProcess),
					},
				},
				Success: true,
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "option",
						Value: string(exitProcess),
					},
				},
				Success: true,
			},
		},
	},

	// Bad value. Mixed casing
	{
		Name:     "invalid_process_mixed_case",
		InputCLI: "Process",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},

	// Correct value (exit thread)
	{
		Name:     "valid_thread",
		InputCLI: "thread",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "option",
						Value: string(exitThread),
					},
				},
				Success: true,
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "option",
						Value: string(exitThread),
					},
				},
			},
		},
	},

	// Bad value. Mixed casing
	{
		Name:     "invalid_thread_mixed_case",
		InputCLI: "Thread",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
}

func TestExitCommand(t *testing.T) {
	cmdtesting.RunTestCases(t, exitCommandMetadata, ExitCommand{}, nil, testCases)
}
