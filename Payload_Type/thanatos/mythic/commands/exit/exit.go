package exit

import (
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
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
	HelpString:          "exit",
	Version:             1,
	MitreAttackMappings: []string{},
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
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: func(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
		return ExitCommand{}.CreateTasking(task)
	},
}

func (c ExitCommand) ParseArgString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	switch input {
	case "process":
		args.SetArgValue("option", string(exitProcess))
	case "thread":
		args.SetArgValue("option", string(exitThread))
	default:
		return fmt.Errorf("%s is an invalid exit option. Option must either be 'process' or 'thread'", input)
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
