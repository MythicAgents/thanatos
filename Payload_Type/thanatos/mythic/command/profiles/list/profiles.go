package agentfunctions

import (
	"encoding/json"
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/logging"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

var profilesCommandDefinition = agentstructs.Command{
	Name:        "profiles",
	Description: "Gets information about the configured C2 profiles.",
	HelpString:  "profiles",
	Version:     1,

	MitreAttackMappings: []string{},

	SupportedUIFeatures: []string{"callback_table:profiles"},

	Author: "@M_alphaaa",

	CommandAttributes: agentstructs.CommandAttribute{
		SupportedOS: []string{
			agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS,
		},
		CommandIsBuiltin: true,
	},

	TaskFunctionProcessResponse: profilesProcessResponse,

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

func profilesProcessResponse(msg agentstructs.PtTaskProcessResponseMessage) agentstructs.PTTaskProcessResponseMessageResponse {
	processResponseMessage := agentstructs.PTTaskProcessResponseMessageResponse{
		TaskID:  msg.TaskData.Task.ID,
		Success: false,
	}

	logging.LogInfo(fmt.Sprintf("Response: %#v", msg.Response))

	response := msg.Response.(string)

	c2ProfileData := []C2Profile{}
	if err := json.Unmarshal([]byte(response), &c2ProfileData); err != nil {
		processResponseMessage.Error = fmt.Sprintf("Failed to deserialize C2 profile JSON: %s", err.Error())
		return processResponseMessage
	}

	output := ""
	for _, profile := range c2ProfileData {
		output += fmt.Sprintf(
			"ID = %d, Name = %s, Enabled = %t, Defunct = %t\n",
			profile.Id,
			profile.Name,
			profile.Enabled,
			profile.Defunct,
		)
	}

	mythicrpc.SendMythicRPCResponseCreate(mythicrpc.MythicRPCResponseCreateMessage{
		TaskID:   msg.TaskData.Task.ID,
		Response: []byte(output),
	})

	extraInfo := ExtraInfoStruct{}
	if err := json.Unmarshal([]byte(msg.TaskData.Callback.ExtraInfo), &extraInfo); err != nil {
		processResponseMessage.Error = fmt.Sprintf("Failed to deserialize extra info JSON: %s", err.Error())
		return processResponseMessage
	}

	extraInfo.C2Profiles = c2ProfileData

	extraInfoBytes, err := json.MarshalIndent(extraInfo, "", " ")
	if err != nil {
		processResponseMessage.Error = fmt.Sprintf("Failed to serialize extra info JSON: %s", err.Error())
		return processResponseMessage
	}

	extraInfoStr := string(extraInfoBytes)

	mythicrpc.SendMythicRPCCallbackUpdate(mythicrpc.MythicRPCCallbackUpdateMessage{
		CallbackID: &msg.TaskData.Callback.ID,
		ExtraInfo:  &extraInfoStr,
	})

	processResponseMessage.Success = true
	return processResponseMessage
}

func init() {
	agentstructs.AllPayloadData.Get("thanatos").AddCommand(profilesCommandDefinition)
}
