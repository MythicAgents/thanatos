package commands

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var getWorkingHoursCommandDefinition = agentstructs.Command{
	Name:        "get-workinghours",
	Description: "Get the currently configured working hours.",
	HelpString:  "get-workinghours",
	Version:     1,

	MitreAttackMappings: []string{},

	SupportedUIFeatures: []string{"callback_table:get-workinghours"},

	Author: "@M_alphaaa",

	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
		CommandIsBuiltin: true,
	},

	ScriptOnlyCommand: true,

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		return nil
	},

	TaskFunctionCreateTasking: getWorkingHoursCreateTasking,
}

func getWorkingHoursFormatTime(t *time.Time) string {
	var output string
	if t.Hour() < 12 {
		output = fmt.Sprintf("%d:%02d am", t.Hour(), t.Minute())
	} else {
		output = t.Format("12:04 pm")
	}

	output += " (" + t.Format(time.TimeOnly)[:5] + ")"
	return output
}

func getWorkingHoursCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	extraInfo := ExtraInfoStruct{}
	err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo)
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse extra info JSON: %s", err.Error())
		return taskResponse
	}

	workingHoursSplit := strings.Split(extraInfo.WorkingHours, "-")

	workingStartStr := workingHoursSplit[0]
	workingEndStr := workingHoursSplit[1]

	workingStart, err := time.Parse(time.TimeOnly, workingStartStr+":00")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse the working start: %s", err.Error())
		return taskResponse
	}

	workingEnd, err := time.Parse(time.TimeOnly, workingEndStr+":00")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse the working end: %s", err.Error())
		return taskResponse
	}

	formattedWorkingStart := getWorkingHoursFormatTime(&workingStart)
	formattedWorkingEnd := getWorkingHoursFormatTime(&workingEnd)

	output := fmt.Sprintf("Agent will only checkin between %s and %s following the host's system time.",
		formattedWorkingStart, formattedWorkingEnd)

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   task.Task.ID,
		Response: []byte(output),
	})

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(getWorkingHoursCommandDefinition)
}
