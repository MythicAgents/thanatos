package agentfunctions

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var workingHourscommandDefinition = agentstructs.Command{
	Name:        "workinghours",
	Description: "Set the agent's working hours.",
	HelpString:  "workinghours [HH:MM] [HH:MM]",
	Version:     1,
	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"workingHoursPostRunActions": workingHoursPostRunActions,
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
			Name:             "start",
			Description:      "Start of the working hours (HH:MM).",
			ModalDisplayName: "Start of the working hours (HH:MM).",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},

		{
			Name:             "end",
			Description:      "End of the working hours (HH:MM).",
			ModalDisplayName: "End of the working hours (HH:MM).",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     2,
				},
			},
		},
	},

	TaskFunctionParseArgString: workingHoursParseArgs,

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: workingHourscreateTasking,
}

func validateWorkingHours(timeString string) error {
	if _, err := time.Parse(time.TimeOnly, timeString+":00"); err != nil {
		return err
	}

	return nil
}

func workingHoursParseArgs(args *agentstructs.PTTaskMessageArgsData, input string) error {
	cmdLineArgs := strings.Split(input, " ")

	if len(cmdLineArgs) != 2 {
		return errors.New("Working hours not provided. (usage: workinghours HH:MM HH:MM)")
	}

	if err := validateWorkingHours(cmdLineArgs[0]); err != nil {
		return errors.New(fmt.Sprintf("Failed to parse working start: %s", err.Error()))
	}

	if err := validateWorkingHours(cmdLineArgs[1]); err != nil {
		return errors.New(fmt.Sprintf("Failed to parse working end: %s", err.Error()))
	}

	args.SetArgValue("start", cmdLineArgs[0])
	args.SetArgValue("end", cmdLineArgs[1])

	return nil
}

func workingHoursPostRunActions(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {

	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	workingStartNum, err := task.Args.GetNumberArg("start")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to get the working start time from the task args: %s", err.Error())
		return completionMessage
	}

	workingStartTime := time.Unix(int64(workingStartNum), 0)

	workingEndNum, err := task.Args.GetNumberArg("end")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to get the working end time from the task args: %s", err.Error())
		return completionMessage
	}

	workingEndTime := time.Unix(int64(workingEndNum), 0)

	formattedWorkingStart := getWorkingHoursFormatTime(&workingStartTime)
	formattedWorkingEnd := getWorkingHoursFormatTime(&workingEndTime)

	output := fmt.Sprintf(
		"Agent will only checkin between %s and %s following the host's system time.",
		formattedWorkingStart,
		formattedWorkingEnd,
	)

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   task.Task.ID,
		Response: []byte(output),
	})

	extraInfo := ExtraInfoStruct{}
	if err = json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to parse extra info JSON: %s", err.Error())
		return completionMessage
	}

	extraInfo.WorkingHours = fmt.Sprintf(
		"%02d:%02d-%02d:%02d",
		workingStartTime.Hour(),
		workingStartTime.Minute(),
		workingEndTime.Hour(),
		workingEndTime.Minute(),
	)

	extraInfoBytes, err := json.MarshalIndent(extraInfo, "", "  ")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to serialize extra info JSON: %s", err.Error())
		return completionMessage
	}

	extraInfoStr := string(extraInfoBytes)

	mythicrpc.SendMythicRPCCallbackUpdate(mythicrpc.MythicRPCCallbackUpdateMessage{
		CallbackID: &task.Task.CallbackID,
		ExtraInfo:  &extraInfoStr,
	})

	completionMessage.Success = true
	return completionMessage
}

func workingHourscreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	workingStartStr, err := task.Args.GetStringArg("start")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to get the working hours start time: %s", err.Error())
		return taskResponse
	}

	workingEndStr, err := task.Args.GetStringArg("end")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to get the working hours end time: %s", err.Error())
		return taskResponse
	}

	workingStartTime, err := time.Parse(time.TimeOnly, workingStartStr+":00")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse the working start time: %s", err.Error())
		return taskResponse
	}

	workingEndTime, err := time.Parse(time.TimeOnly, workingEndStr+":00")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse the working end time: %s", err.Error())
		return taskResponse
	}

	if workingStartTime.After(workingEndTime) {
		taskResponse.Error = "Invalid working hours. Start time is after the end time."
		return taskResponse
	}

	task.Args.RemoveArg("start")
	task.Args.AddArg(agentstructs.CommandParameter{
		Name:          "start",
		ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
	})

	task.Args.SetArgValue(
		"start",
		float64((workingStartTime.Hour()*3600)+(workingStartTime.Minute()*60)),
	)

	task.Args.RemoveArg("end")
	task.Args.AddArg(agentstructs.CommandParameter{
		Name:          "end",
		ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
	})

	task.Args.SetArgValue(
		"end",
		float64((workingEndTime.Hour()*3600)+(workingEndTime.Minute()*60)),
	)

	displayParams := fmt.Sprintf(
		"start = %02d:%02d, end = %02d:%02d",
		workingStartTime.Hour(),
		workingStartTime.Minute(),
		workingEndTime.Hour(),
		workingEndTime.Minute(),
	)

	taskResponse.DisplayParams = &displayParams

	completionFunction := "workingHoursPostRunActions"
	taskResponse.CompletionFunctionName = &completionFunction

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(workingHourscommandDefinition)
}
