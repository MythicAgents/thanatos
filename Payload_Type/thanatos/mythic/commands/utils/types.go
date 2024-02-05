package utils

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

type CommandTaskFunction interface {
	ParseArgString(args *agentstructs.PTTaskMessageArgsData, input string) error
	CreateTasking(
		task *agentstructs.PTTaskMessageAllData,
	) agentstructs.PTTaskCreateTaskingMessageResponse
}

type CommandPostRunFunction interface {
	PostRunAction(
		task *agentstructs.PTTaskMessageAllData,
		data *agentstructs.PTTaskMessageAllData,
		subtask *agentstructs.SubtaskGroupName,
	) agentstructs.PTTaskCompletionFunctionMessageResponse
}

type RPCExecutor interface {
	SendMythicRPCCallbackUpdate(
		input mythicrpc.MythicRPCCallbackUpdateMessage,
	) (*mythicrpc.MythicRPCCallbackUpdateMessageResponse, error)

	SendMythicRPCResponseCreate(
		input mythicrpc.MythicRPCResponseCreateMessage,
	) (*mythicrpc.MythicRPCResponseCreateMessageResponse, error)
}
