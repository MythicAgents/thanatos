package exit

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	thanatoserrors "thanatos/errors"
)

type exitOption string

const (
	exitProcess exitOption = "process"
	exitThread  exitOption = "thread"
)

type ExitCommand struct{}

var exitCommandMetadata = agentstructs.Command{
	Name:                "exit",
	Description:         "Exit the current session and kill the agent.",
	HelpString:          "exit [thread/process]",
	Version:             1,
	MitreAttackMappings: []string{},
	SupportedUIFeatures: []string{"callback_table:exit"},
	Author:              "@M_alphaaa",
	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
		CommandIsBuiltin: true,
	},

	CommandParameters: []agentstructs.CommandParameter{
		{
			Name:             "option",
			Description:      "Exit option",
			ModalDisplayName: "Exit option",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_CHOOSE_ONE,
			Choices: []string{
				string(exitProcess),
				string(exitThread),
			},
			DefaultValue: string(exitThread),
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		return ExitCommand{}.ParseArgString(args, input)
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		return args.LoadArgsFromDictionary(input)
	},

	TaskFunctionCreateTasking: func(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
		return ExitCommand{}.CreateTasking(task)
	},
}

func (c ExitCommand) ParseArgString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	switch input {
	case "process":
		if err := args.SetArgValue("option", string(exitProcess)); err != nil {
			return thanatoserrors.Errorf("failed to set option parameter: %s", err.Error())
		}
	case "thread":
		if err := args.SetArgValue("option", string(exitThread)); err != nil {
			return thanatoserrors.Errorf("failed to set option parameter: %s", err.Error())
		}
	default:
		return thanatoserrors.Errorf("%s is an invalid exit option. Option must either be 'process' or 'thread'", input)
	}
	return nil
}

func (c ExitCommand) CreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	return agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: true,
		TaskID:  task.Task.ID,
	}
}

func Initialize() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(exitCommandMetadata)
}
