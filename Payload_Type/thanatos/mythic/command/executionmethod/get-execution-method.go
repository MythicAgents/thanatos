package agentfunctions

import (
	"encoding/json"
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var getExecutionMethodCommandDefinition = agentstructs.Command{
	Name:        "get-execution-method",
	Description: "Get the method for executing loaded commands or modules.",
	HelpString:  "get-execution-method",
	Version:     1,

	MitreAttackMappings: []string{},

	SupportedUIFeatures: []string{"callback_table:get-execution-method"},

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

	TaskFunctionCreateTasking: getExecutionMethodCreateTasking,
}

func getExecutionMethodCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
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

	output := ""
	if extraInfo.ExecInternal {
		output = "internal"
	} else {
		output = "external"
	}

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   task.Task.ID,
		Response: []byte(output),
	})

	completed := true
	taskResponse.Completed = &completed

	status := "success"
	taskResponse.TaskStatus = &status

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(getExecutionMethodCommandDefinition)
}
