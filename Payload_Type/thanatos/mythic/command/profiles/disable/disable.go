package agentfunctions

import (
	"encoding/json"
	"errors"
	"fmt"
	"strconv"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
	"golang.org/x/exp/slices"
)

var disableProfileCommandDefinition = agentstructs.Command{
	Name:        "disable-profile",
	Description: "Disables a C2 profile in the agent.",
	HelpString:  "disable-profile [profile id]",
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
			Name:             "id",
			Description:      "ID of the profile to disable.",
			ModalDisplayName: "ID of the profile to disable.",
			ParameterType:    agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
			ParameterGroupInformation: []agentstructs.ParameterGroupInfo{
				{
					ParameterIsRequired: true,
					UIModalPosition:     1,
				},
			},
		},
	},

	TaskCompletionFunctions: map[string]agentstructs.PTTaskCompletionFunction{
		"disableProfilePostRunActions": disableProfilePostRunActions,
	},

	TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
		return nil
	},

	TaskFunctionParseArgDictionary: func(args *agentstructs.PTTaskMessageArgsData, input map[string]interface{}) error {
		args.LoadArgsFromDictionary(input)
		return nil
	},

	TaskFunctionCreateTasking: disableProfileCreateTasking,
}

func disableProfilePostRunActions(
	task *agentstructs.PTTaskMessageAllData,
	data *agentstructs.PTTaskMessageAllData,
	subtask *agentstructs.SubtaskGroupName,
) agentstructs.PTTaskCompletionFunctionMessageResponse {
	completionMessage := agentstructs.PTTaskCompletionFunctionMessageResponse{
		TaskID:       task.Task.ID,
		ParentTaskId: 0,
		Success:      false,
	}

	profileID, err := task.Args.GetNumberArg("id")
	if err != nil {
		completionMessage.Error = "Failed to find profile id from task arguments"
		return completionMessage
	}

	output := fmt.Sprintf("Disabled profile %d.", int(profileID))

	mythicrpc.SendMythicRPCResponseCreate(
		mythicrpc.MythicRPCResponseCreateMessage{
			TaskID:   task.Task.ID,
			Response: []byte(output),
		},
	)

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		completionMessage.Error = fmt.Sprintf("Failed to deserialize extra info JSON: %s", err.Error())
		return completionMessage
	}

	profileIndex := slices.IndexFunc(extraInfo.C2Profiles, func(profile C2Profile) bool {
		return profile.Id == int(profileID)
	})

	if profileIndex == -1 {
		completionMessage.Error = "Failed to find C2 profile in extra info"
		return completionMessage
	}

	extraInfo.C2Profiles[profileIndex].Enabled = false

	extraInfoBytes, err := json.MarshalIndent(extraInfo, "", "  ")
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

func disableProfileParseArgsString(args *agentstructs.PTTaskMessageArgsData, input string) error {
	profileID, err := strconv.Atoi(input)
	if err != nil {
		return errors.New(fmt.Sprintf("Failed to parse profile ID: %s", err.Error()))
	}

	args.SetArgValue("id", float64(profileID))
	return nil
}

func disableProfileCreateTasking(task *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
	taskResponse := agentstructs.PTTaskCreateTaskingMessageResponse{
		Success: false,
		TaskID:  task.Task.ID,
	}

	profileID, err := task.Args.GetNumberArg("id")
	if err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to get profile ID: %s", err.Error())
		return taskResponse
	}

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(task.Callback.ExtraInfo), &extraInfo); err != nil {
		taskResponse.Error = fmt.Sprintf("Failed to deserialize extra info: %s", err.Error())
		return taskResponse
	}

	if len(extraInfo.C2Profiles) == 1 {
		taskResponse.Error = "There is only one C2 profile configured in the agent. Disabling it will kill the callback."

		mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
			TaskID:   task.Task.ID,
			Response: []byte(taskResponse.Error),
		})

		return taskResponse
	}

	profileIndex := slices.IndexFunc(extraInfo.C2Profiles, func(profile C2Profile) bool {
		return profile.Id == int(profileID)
	})

	if profileIndex == -1 {
		taskResponse.Error = fmt.Sprintf("Could not find C2 profile with the ID: %d", int(profileID))

		mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
			TaskID:   task.Task.ID,
			Response: []byte(taskResponse.Error),
		})

		return taskResponse
	}

	if !extraInfo.C2Profiles[profileIndex].Enabled {
		output := "Profile already disabled."

		mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
			TaskID:   task.Task.ID,
			Response: []byte(output),
		})

		taskResponse.Success = true

		completed := true
		taskResponse.Completed = &completed

		status := "success"
		taskResponse.TaskStatus = &status
		return taskResponse
	}

	completionFunction := "disableProfilePostRunActions"
	taskResponse.CompletionFunctionName = &completionFunction

	displayParams := fmt.Sprint(int(profileID))
	taskResponse.DisplayParams = &displayParams

	taskResponse.Success = true
	return taskResponse
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(disableProfileCommandDefinition)
}
