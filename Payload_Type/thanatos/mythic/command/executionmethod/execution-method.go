package agentfunctions

import (
	"encoding/json"
	"fmt"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var executionMethodCommandDefinition = agentstructs.Command{
	Name:        "execution-method",
	Description: "Set the method for executing loaded commands or modules.",
	HelpString:  "execution-method [internal/external]",
	Version:     1,

	MitreAttackMappings: []string{},

	Author: "@M_alphaaa",

	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
		CommandIsBuiltin: true,
	},

	CommandParameters: []agentstructs.CommandParameter{
		{
			Name:             "method",
			Description:      "Method for executing loaded commands. [internal/external]",
			ModalDisplayName: "Method for executing loaded commands. [internal/external]",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_CHOOSE_ONE,
			Choices: []string{
				"internal",
				"external",
			},
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},
	},

	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"executionMethodPostRunActions": executionMethodPostRunActions,
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		args.SetArgValue("method", input)
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: enableProfileCreateTasking,
}

func executionMethodPostRunActions(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {
	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	method, err := task.Args.GetStringArg("method")
	if err != nil {
		completionMessage.Error = "Failed to get method argument"
		return completionMessage
	}

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to deserialize extra info JSON: %s", err.Error())
		return completionMessage
	}

	output := ""
	if method == "internal" {
		output = "Agent set to execute loaded commands internally."
		extraInfo.ExecInternal = true
	} else {
		spawnToArgs := strings.Join(extraInfo.Spawnto.Args, " ")

		output = fmt.Sprintf(
			"Agent set to execute loaded commands externally using the configured sapwnto value of '%s",
			extraInfo.Spawnto.Path,
		)

		if len(spawnToArgs) > 0 {
			output += fmt.Sprintf(" %s'.", spawnToArgs)
		} else {
			output += "'."
		}

		extraInfo.ExecInternal = false
	}

	mythicrpc.SendMythicRPCResponseCreate(
		mythicrpc.MythicRPCResponseCreateMessage{
			TaskID:   task.Task.ID,
			Response: []byte(output),
		},
	)

	extraInfoBytes, err := json.MarshalIndent(extraInfo, "", " ")
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

func executionMethodCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	method, err := task.Args.GetStringArg("method")
	if err != nil {
		taskResponse.Error = "Failed to get method argument"
		return taskResponse
	}

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to deserialize extra info JSON: %s", err.Error())
		return taskResponse
	}

	switch method {
	case "internal":
		if extraInfo.ExecInternal {
			mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
				TaskID:   task.Task.ID,
				Response: []byte("Agent already set to execute loaded commands internally."),
			})

			taskResponse.Success = true

			completed := true
			taskResponse.Completed = &completed

			status := "success"
			taskResponse.TaskStatus = &status
			return taskResponse
		}
	case "external":
		if !extraInfo.ExecInternal {
			mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
				TaskID:   task.Task.ID,
				Response: []byte("Agent already set to execute loaded commands externally."),
			})

			taskResponse.Success = true

			completed := true
			taskResponse.Completed = &completed

			status := "success"
			taskResponse.TaskStatus = &status
			return taskResponse
		}

		if extraInfo.Spawnto == nil {
			mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
				TaskID:   task.Task.ID,
				Response: []byte("No spawnto value configured. Set the spawnto value before changing the executio nmethod to external."),
			})

			completed := true
			taskResponse.Completed = &completed

			status := "error"
			taskResponse.TaskStatus = &status
			return taskResponse
		}
	default:
		taskResponse.Error = "Invalid option. Must be either 'internal' or 'external'"
		mythicrpc.SendMythicRPCResponseCreate(
			mythicrpc.MythicRPCResponseCreateMessage{
				TaskID:   task.Task.ID,
				Response: []byte(taskResponse.Error),
			},
		)

		return taskResponse
	}

	taskResponse.DisplayParams = &method

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(executionMethodCommandDefinition)
}
