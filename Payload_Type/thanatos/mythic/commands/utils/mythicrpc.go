package utils

import "github.com/MythicMeta/MythicContainer/mythicrpc"

type MythicRPCExecutor struct{}

func (rpc MythicRPCExecutor) SendMythicRPCCallbackUpdate(
	input mythicrpc.MythicRPCCallbackUpdateMessage,
) (*mythicrpc.MythicRPCCallbackUpdateMessageResponse, error) {
	return mythicrpc.SendMythicRPCCallbackUpdate(input)
}

func (rpc MythicRPCExecutor) SendMythicRPCResponseCreate(
	input mythicrpc.MythicRPCResponseCreateMessage,
) (*mythicrpc.MythicRPCResponseCreateMessageResponse, error) {
	return mythicrpc.SendMythicRPCResponseCreate(input)
}
