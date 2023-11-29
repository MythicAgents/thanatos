package agentfunctions

import (
	"errors"
	"fmt"
	"strconv"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var sleepCommandDefinition = agentstructs.Command{
	Name:        "sleep",
	Description: "Change the agent's sleep interval. Suffix can either be [s, m, h]",
	HelpString:  "sleep [number][suffix] [jitter]",
	Version:     1,
	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"sleepPostRunActions": sleepPostRunActions,
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

	TaskFunctionParseArgString: sleepParseArgsString,

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: sleepCreateTasking,
}

func sleepParseArgsString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	cmdLineArgs := strings.Split(input, " ")

	args.SetArgValue("interval", cmdLineArgs[0])

	if len(cmdLineArgs) > 1 {
		jitter, err := strconv.Atoi(cmdLineArgs[1])
		if err != nil {
			return err
		}

		args.SetArgValue("jitter", jitter)
	} else {
		args.SetArgValue("jitter", 0)
	}

	return nil
}

func sleepUpdateSleepInfo(callbackID int, output string) error {
	if _, err := mythicrpc.SendMythicRPCCallbackUpdate(mythicrpc.MythicRPCCallbackUpdateMessage{
		CallbackID: &callbackID,
		SleepInfo:  &output,
	}); err != nil {
		return errors.New(fmt.Sprintf("Failed to update sleep info: %s", err.Error()))
	}

	return nil
}

func sleepFormulateOutput(taskID int, output string) error {
	if _, err := mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   taskID,
		Response: []byte(output),
	}); err != nil {
		return errors.New(fmt.Sprintf("Failed to set the task output: %s", err.Error()))
	}

	return nil
}

func sleepPostRunActions(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {

	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	interval, err := task.Args.GetNumberArg("interval")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to get sleep interval in post processing: %s", err.Error())
		return completionMessage
	}

	jitter, err := task.Args.GetNumberArg("jitter")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to get sleep jitter in post processing: %s", err.Error())
		return completionMessage
	}

	var sleepInfo string
	if jitter == 0 {
		sleepInfo = fmt.Sprintf("Agent will checkin every %d second(s)", int(interval))
	} else {
		jitter /= 100
		startRange := interval - (float64(interval) * jitter)
		endRange := interval + (float64(interval) * jitter)

		sleepInfo = fmt.Sprintf("Agent will checkin between %d and %d second(s)", int(startRange), int(endRange))
	}

	if err := sleepFormulateOutput(task.Task.ID, sleepInfo); err != nil {
		completionMessage.Error = err.Error()
		return completionMessage
	}

	if err := sleepUpdateSleepInfo(task.Task.CallbackID, sleepInfo); err != nil {
		completionMessage.Error = err.Error()
		return completionMessage
	}

	completionMessage.Success = true
	return completionMessage
}

func sleepCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	intervalStr, err := task.Args.GetStringArg("interval")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to get interval argument: %s", err.Error())
		return taskResponse
	}

	jitter, err := task.Args.GetNumberArg("jitter")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to get jitter argument: %s", err.Error())
		return taskResponse
	}

	var timeUnit string
	conversionFactor := 1
	realInterval, err := strconv.Atoi(intervalStr)
	if err != nil {
		realInterval, err = strconv.Atoi(intervalStr[:len(intervalStr)-1])
		if err != nil {
			taskResponse.Error = fmt.Sprintf("Failed to parse the interval as an integer: %s", err.Error())
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
			taskResponse.Error = fmt.Sprintf("Invalid interval suffix of '%s'. Must be either s, m, or h", timeUnit)
			return taskResponse
		}

		realInterval *= conversionFactor
	} else {
		timeUnit = "s"
	}

	if jitter < 0 {
		taskResponse.Error = "Jitter cannot be negative"
		return taskResponse
	}

	task.Args.RemoveArg("interval")
	task.Args.AddArg(agentstructs.CommandParameter{
		Name:          "interval",
		ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
	})

	task.Args.SetArgValue("interval", float64(realInterval))

	displayInterval := int(realInterval / conversionFactor)
	displayParams := fmt.Sprintf("interval = %d%s, jitter = %d%%", displayInterval, timeUnit, int(jitter))
	taskResponse.DisplayParams = &displayParams

	completionFunction := "sleepPostRunActions"
	taskResponse.CompletionFunctionName = &completionFunction

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(sleepCommandDefinition)
}
