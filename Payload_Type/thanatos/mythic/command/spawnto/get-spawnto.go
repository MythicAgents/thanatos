package agentfunctions

import (
	"encoding/json"
	"fmt"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var getSpawnToCommandDefinition = agentstructs.Command{
	Name:        "get-spawnto",
	Description: "Get the configured spawnto value.",
	HelpString:  "profiles",
	Version:     1,

	MitreAttackMappings: []string{},

	SupportedUIFeatures: []string{"callback_table:get-spawnto"},

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

	TaskFunctionCreateTasking: getSpawnToCreateTasking,
}

func getSpawnToCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to parse the extra info JSON: %s", err.Error())
		return taskResponse
	}

	output := ""
	if extraInfo.Spawnto != nil {
		spawnToPath := extraInfo.Spawnto.Path
		spawnToArgs := strings.Join(extraInfo.Spawnto.Args, " ")

		output = fmt.Sprintf("Spawnto value set to '%s", spawnToPath)
		if len(spawnToArgs) > 0 {
			output += fmt.Sprintf(" %s'", spawnToArgs)
		} else {
			output += "'"
		}
	} else {
		output = "Spawnto value not set."
	}

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   task.Task.ID,
		Response: []byte(output),
	})

	completed := true
	taskResponse.Completed = &completed

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(getSpawnToCommandDefinition)
}
