package agentfunctions

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
	"golang.org/x/exp/slices"
)

var spawnToCommandDefinition = agentstructs.Command{
	Name:        "spawnto",
	Description: "Change the configured spawnto value.",
	HelpString:  "spawnto [path] [args...]",
	Version:     1,
	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"spawnToPostRunActions": spawnToPostRunActions,
	},

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
			Name:             "path",
			Description:      "Path for spawnto.",
			ModalDisplayName: "Path for spawnto.",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},

		{
			Name:             "args",
			Description:      "Command line arguments for spawnto.",
			ModalDisplayName: "Command line arguments for spawnto.",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_STRING,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: false,
					UIModalPosition:     2,
				},
			},
		},
	},

	TaskFunctionParseArgString: spawnToParseArgsString,

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: spawnToCreateTasking,
}

func spawnToParseArgsString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	cmdLineArgs := strings.Split(input, " ")

	if len(cmdLineArgs) == 0 {
		return errors.New("Not enough arguments.")
	}

	args.SetArgValue("path", cmdLineArgs[0])

	if len(cmdLineArgs) > 1 {
		spawnToArgs := strings.Join(cmdLineArgs[1:], " ")
		args.SetArgValue("args", spawnToArgs)
	}

	return nil
}

func spawnToPostRunActions(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {
	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	spawnToPath, err := task.Args.GetStringArg("path")
	if err != nil {
		completionMessage.Error = "Failed to get the spawnto path"
		return completionMessage
	}

	spawnToArgs, err := task.Args.GetStringArg("args")
	if err != nil {
		spawnToArgs = ""
	}

	spawnToArgsSplit := strings.Split(spawnToArgs, " ")

	output := ""
	if len(spawnToArgsSplit) > 0 {
		output = fmt.Sprintf("Set spawnto value to '%s %s'", spawnToPath, spawnToArgs)
	} else {
		output = fmt.Sprintf("Set spawnto value to '%s'", spawnToPath)
	}

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   task.Task.ID,
		Response: []byte(output),
	})

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to deserialize the extra info JSON: %s", err.Error())
		return completionMessage
	}

	if extraInfo.Spawnto == nil {
		extraInfo.Spawnto = &SpawnToValue{
			Path: spawnToPath,
			Args: spawnToArgsSplit,
		}
	} else {
		extraInfo.Spawnto.Path = spawnToPath
		extraInfo.Spawnto.Args = spawnToArgsSplit
	}

	extraInfoBytes, err := json.MarshalIndent(extraInfo, "", " ")
	if err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to serialize extra info JSON: %s", err.Error())
		return completionMessage
	}

	extraInfoStr := string(extraInfoBytes)

	mythicrpc.SendMythicRPCCallbackUpdate(mythicrpc.MythicRPCCallbackUpdateMessage{
		CallbackID: &task.Callback.ID,
		ExtraInfo:  &extraInfoStr,
	})

	completionMessage.Success = true
	return completionMessage
}

func spawnToCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	spawnToPath, err := task.Args.GetStringArg("path")
	if err != nil {
		taskResponse.Error = "Failed to get path argument"
		return taskResponse
	}

	spawnToArgs, err := task.Args.GetStringArg("args")
	if err != nil {
		spawnToArgs = ""
	}

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to deserialize extra info JSON: %s", err.Error())
		return taskResponse
	}

	spawnToArgsSlice := strings.Split(spawnToArgs, " ")

	if extraInfo.Spawnto != nil {
		output := ""
		if extraInfo.Spawnto.Path == spawnToPath && slices.Compare(extraInfo.Spawnto.Args, spawnToArgsSlice) == 0 {
			if len(spawnToArgsSlice) > 0 {
				output = fmt.Sprintf("Spawnto value already set to '%s %s'", spawnToPath, spawnToArgs)
			} else {
				output = fmt.Sprintf("Spawnto value already set to '%s'", spawnToPath)
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
	}

	displayParams := spawnToPath
	if len(spawnToArgs) > 0 {
		displayParams += " " + spawnToArgs
	}

	taskResponse.DisplayParams = &displayParams

	completionFunction := "spawnToPostRunActions"
	taskResponse.CompletionFunctionName = &completionFunction

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(spawnToCommandDefinition)
}
