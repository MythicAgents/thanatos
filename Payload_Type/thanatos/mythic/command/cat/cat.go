package cat

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(agentstructs.Command{
		Name:        "cat",
		Description: "Display the contents of a file.",
		HelpString:  "cat [file path]",
		Version:     1,

		MitreAttackMappings: []string{
			"T1005",
			"T1039",
			"T1025",
		},

		Author: "@M_alphaaa",

		CommandAttributes: agentstructs.CommandAttribute{
			SupportedOS: []string{
				agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
			},
		},

		CommandParameters: []agentstructs.CommandParameter{
			{
				Name:             "file",
				Description:      "File to display the contents of.",
				ModalDisplayName: "File to display the contents of.",
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
			args.SetArgValue("file", input)
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

			catFile, _ := task.Args.GetStringArg("file")
			response.DisplayParams = &catFile
			return response
		},
	})
}
