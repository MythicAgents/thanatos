package sleep

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/MythicAgents/thanatos/pkg/commands/utils"
	thanatoserror "github.com/MythicAgents/thanatos/pkg/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

type SleepCommand struct {
	rpc utils.RPCExecutor
}

var sleepCommandMetadata = agentstructs.Command{
	Name:        "sleep",
	Description: "Change the agent's sleep interval. Suffix can either be [s, m, h]",
	HelpString:  "sleep [number][suffix] [jitter]",
	Version:     1,
	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"sleepPostRunActions": func(
			task *agentstructs.PTTaskMessageAllData,
			data *agentstructs.PTTaskMessageAllData,
			subtask *agentstructs.SubtaskGroupName,
		) agentstructs.PTTaskCompletionFunctionMessageResponse {
			return SleepCommand{
				rpc: &utils.MythicRPCExecutor{},
			}.PostRunAction(task, data, subtask)
		},
	},

	MitreAttackMappings: []string{
		"T1029",
	},

	Author: "@M_alphaaa",
	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
		CommandIsBuiltin: true,
	},

	CommandParameters: []agentstructs.CommandParameter{
		{
			Name:             "interval",
			Description:      "Interval to sleep.",
			ModalDisplayName: "Interval to sleep.",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},

		{
			Name:             "jitter",
			Description:      "Sleep jitter.",
			ModalDisplayName: "Sleep jitter.",
			DefaultValue:     0,
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: false,
					UIModalPosition:     2,
				},
			},
		},
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		return SleepCommand{
			rpc: &utils.MythicRPCExecutor{},
		}.ParseArgString(args, input)
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		return args.LoadArgsFromDictionary(input)
	},

	TaskFunctionCreateTasking: func(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
		return SleepCommand{
			rpc: &utils.MythicRPCExecutor{},
		}.CreateTasking(task)
	},
}

func (c SleepCommand) ParseArgString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	cmdLineArgs := strings.Split(input, " ")

	if err := args.SetArgValue("interval", cmdLineArgs[0]); err != nil {
		return thanatoserror.Errorf("failed to set interval value: %s", err.Error())
	}

	if len(cmdLineArgs) > 1 {
		jitter, err := strconv.Atoi(cmdLineArgs[1])
		if err != nil {
			return thanatoserror.Errorf("failed to parse the sleep jitter value from the command line: %s", err)
		}

		if err := args.SetArgValue("jitter", jitter); err != nil {
			return thanatoserror.Errorf("failed to set jitter value: %s", err.Error())
		}
	} else {
		if err := args.SetArgValue("jitter", 0); err != nil {
			return thanatoserror.Errorf("failed to set jitter value to 0: %s", err.Error())
		}
	}

	return nil
}

func (c SleepCommand) CreateTasking(
	task *agentstructs.PTTaskMessageAllData,
) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	intervalStr, err := task.Args.GetStringArg("interval")
	if err != nil {
		taskResponse.Error = thanatoserror.Errorf("failed to get interval argument: %s", err.Error()).Error()
		return taskResponse
	}

	jitter, err := task.Args.GetNumberArg("jitter")
	if err != nil {
		taskResponse.Error = thanatoserror.Errorf("failed to get jitter argument: %s", err.Error()).Error()
		return taskResponse
	}

	var timeUnit string
	conversionFactor := 1
	realInterval, err := strconv.Atoi(intervalStr)
	if err != nil {
		realInterval, err = strconv.Atoi(intervalStr[:len(intervalStr)-1])
		if err != nil {
			taskResponse.Error = thanatoserror.Errorf("failed to parse the interval as an integer: %s", err.Error()).Error()
			return taskResponse
		}

		timeUnit = intervalStr[len(intervalStr)-1:]

		switch timeUnit {
		case "s":
			conversionFactor = 1
		case "m":
			conversionFactor = 60
		case "h":
			conversionFactor = 3600
		default:
			taskResponse.Error = thanatoserror.Errorf("invalid interval suffix of '%s'. Must be either s, m, or h", timeUnit).Error()
			return taskResponse
		}

		realInterval *= conversionFactor
	} else {
		timeUnit = "s"
	}

	if realInterval < 0 {
		taskResponse.Error = thanatoserror.New("interval cannot be negative").Error()
		return taskResponse
	}

	if jitter < 0 {
		taskResponse.Error = thanatoserror.New("jitter cannot be negative").Error()
		return taskResponse
	}

	if err := task.Args.RemoveArg("interval"); err != nil {
		taskResponse.Error = thanatoserror.Errorf("failed to remove existing interval parameter: %s", err.Error()).Error()
		return taskResponse
	}

	if err := task.Args.AddArg(agentstructs.CommandParameter{
		Name:          "interval",
		ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
	}); err != nil {
		taskResponse.Error = thanatoserror.Errorf("failed to create interval parameter: %s", err.Error()).Error()
		return taskResponse
	}

	if err := task.Args.SetArgValue("interval", float64(realInterval)); err != nil {
		taskResponse.Error = thanatoserror.Errorf("failed to set interval parameter value: %s", err.Error()).Error()
		return taskResponse
	}

	displayInterval := int(realInterval / conversionFactor)
	displayParams := fmt.Sprintf("interval = %d%s, jitter = %d%%", displayInterval, timeUnit, int(jitter))
	taskResponse.DisplayParams = &displayParams

	completionFunction := "sleepPostRunActions"
	taskResponse.CompletionFunctionName = &completionFunction

	taskResponse.Success = true
	return taskResponse
}

func (c SleepCommand) PostRunAction(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {
	response := agentstructs.PTTaskCompletionFunctionMessageResponse{
		Success: true,
		TaskID:  task.Task.ID,
	}

	return response
}

func Initialize() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(sleepCommandMetadata)
}
