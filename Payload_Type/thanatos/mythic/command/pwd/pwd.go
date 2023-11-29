package commands

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

var pwdCommandDefinition = agentstructs.Command{
	Name:        "pwd",
	Description: "Print the current working directory.",
	HelpString:  "pwd",
	Version:     1,

	MitreAttackMappings: []string{
		"T1005",
		"T1039",
		"T1025",
	},

	Author: "@M_alphaaa",

	SupportedUIFeatures: []string{"callback_table:pwd"},

	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		return nil
	},

	TaskFunctionCreateTasking: func(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
		response := agentstructs.PTTaskCreateTaskingMessageResponse{
			Success: true,
			TaskID:  task.Task.ID,
		}

		return response
	},
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(pwdCommandDefinition)
}
