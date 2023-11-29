package cd

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

var definition = agentstructs.Command{
	Name:        "cd",
	Description: "Change the current directory of the agent.",
	HelpString:  "cd [directory]",
	Version:     1,

	Author: "@M_alphaaa",

	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
	},

	CommandParameters: []agentstructs.CommandParameter{
		{
			Name:             "directory",
			Description:      "Directory to cd into.",
			ModalDisplayName: "Directory to cd into.",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		args.SetArgValue("directory", input)
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: func(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
		response := agentstructs.PTTaskCreateTaskingMessageResponse{
			Success: true,
			TaskID:  task.Task.ID,
		}

		directory, _ := task.Args.GetStringArg("directory")
		response.DisplayParams = &directory
		return response
	},
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(definition)
}
