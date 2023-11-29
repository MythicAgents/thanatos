package agentfunctions

import (
	"fmt"
	"os"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

var loadCommandDefinition = agentstructs.Command{
	Name:        "load",
	Description: "Load a command into the agent.",
	HelpString:  "load [command]",
	Version:     1,

	MitreAttackMappings: []string{
		"T1620",
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
			Name:                 "command",
			Description:          "Command to load into the agent.",
			ModalDisplayName:     "Command to load into the agent.",
			ParameterType:        agentstructs.COMMAND_PARAMETER_TYPE_CHOOSE_ONE,
			DynamicQueryFunction: loadGetUnloadedCommands,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},
	},

	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"loadAddCallbackCommand": loadAddCallbackCommand,
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		args.SetArgValue("command", input)
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: loadCreateTasking,
}

func loadGetUnloadedCommands(msg agentstructs.PTRPCDynamicQueryFunctionMessage) []string {
	return []string{}
}

func loadAddCallbackCommand(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {
	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	completionMessage.Success = true
	return completionMessage
}

func loadCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	commandName, err := task.Args.GetStringArg("command")
	if err != nil {
		taskResponse.Error = "Failed to get command from task arguments"
		return taskResponse
	}

	modulePath := "/opt/modules/" + commandName

	moduleDir, err := os.ReadDir(modulePath)
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to find module for command '%s': %s", commandName, err.Error())
		return taskResponse
	}

	callbackOs := ""
	if strings.Contains(task.Callback.OS, "Linux") {
		callbackOs = agentstructs.SUPPORTED_OS_LINUX
	} else {
		callbackOs = agentstructs.SUPPORTED_OS_WINDOWS
	}

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(loadCommandDefinition)
}
