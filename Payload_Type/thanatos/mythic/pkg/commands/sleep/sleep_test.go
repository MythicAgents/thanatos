package sleep

import (
	"fmt"
	"math"
	"testing"

	cmdtesting "github.com/MythicAgents/thanatos/pkg/commands/testing"
)

var testCases = []cmdtesting.TestCase{
	{
		Name:     "valid_no_suffix_no_jitter",
		InputCLI: "10",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: float64(10),
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
		},

		ResponseProcessing: cmdtesting.TestCaseResponseProcessingValues{
			CompletionFunctionResult: cmdtesting.TestCaseCompletionFunctionResult{
				Success: true,
			},
			RPCResults: cmdtesting.TestCaseRPCValues{
				Response: "Agent will check in every 10 second(s)",
			},
		},
	},
	{
		Name:     "valid_no_suffix_with_jitter",
		InputCLI: "10 23",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10",
					},
					{
						Name:  "jitter",
						Value: 23,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: float64(10),
					},
					{
						Name:  "jitter",
						Value: 23,
					},
				},
			},
		},

		ResponseProcessing: cmdtesting.TestCaseResponseProcessingValues{
			CompletionFunctionResult: cmdtesting.TestCaseCompletionFunctionResult{
				Success: true,
			},
			RPCResults: cmdtesting.TestCaseRPCValues{
				Response: fmt.Sprintf("Agent will check in between %d and %d second(s)", int(math.Round(10-(10*.23))), int(math.Round(10+(10*.23)))),
			},
		},
	},
	{
		Name:     "invalid_garbage_data",
		InputCLI: "foo",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "foo",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
	{
		Name:     "invalid_garbage_data_with_spaces",
		InputCLI: "foo bar",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
	{
		Name:     "invalid_jitter_negative",
		InputCLI: "10 -2",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10",
					},
					{
						Name:  "jitter",
						Value: -2,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
	{
		Name:     "valid_seconds_suffix",
		InputCLI: "10s",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10s",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: float64(10),
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
		},

		ResponseProcessing: cmdtesting.TestCaseResponseProcessingValues{
			CompletionFunctionResult: cmdtesting.TestCaseCompletionFunctionResult{
				Success: true,
			},
			RPCResults: cmdtesting.TestCaseRPCValues{
				Response: "Agent will check in every 10 second(s)",
			},
		},
	},
	{
		Name:     "valid_minutes_suffix",
		InputCLI: "10m",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10m",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: float64(10 * 60),
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
		},

		ResponseProcessing: cmdtesting.TestCaseResponseProcessingValues{
			CompletionFunctionResult: cmdtesting.TestCaseCompletionFunctionResult{
				Success: true,
			},
			RPCResults: cmdtesting.TestCaseRPCValues{
				Response: fmt.Sprintf("Agent will check in every %d second(s)", 10*60),
			},
		},
	},
	{
		Name:     "valid_hours_suffix",
		InputCLI: "10h",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10h",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: float64(10 * 60 * 60),
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
		},

		ResponseProcessing: cmdtesting.TestCaseResponseProcessingValues{
			CompletionFunctionResult: cmdtesting.TestCaseCompletionFunctionResult{
				Success: true,
			},
			RPCResults: cmdtesting.TestCaseRPCValues{
				Response: fmt.Sprintf("Agent will check in every %d second(s)", 10*60*60),
			},
		},
	},
	{
		Name:     "invalid_negative_interval",
		InputCLI: "-123",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "-123",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
	{
		Name:     "invalid_bad_suffix",
		InputCLI: "10b",
		PreTasking: cmdtesting.TestCaseExpectedValues{
			ParseArgsResult: cmdtesting.TestCaseResultValue{
				Success: true,
				ArgValues: []cmdtesting.TestCaseArgValue{
					{
						Name:  "interval",
						Value: "10b",
					},
					{
						Name:  "jitter",
						Value: 0,
					},
				},
			},
			CreateTaskingResult: cmdtesting.TestCaseResultValue{
				Success: false,
			},
		},
	},
}

func TestSleepCommand(t *testing.T) {
	cmdFunctions := SleepCommand{
		rpc: &cmdtesting.MockRPCExecutor{},
	}

	cmdtesting.RunTestCases(t, sleepCommandMetadata, cmdFunctions, cmdFunctions, testCases)
}
