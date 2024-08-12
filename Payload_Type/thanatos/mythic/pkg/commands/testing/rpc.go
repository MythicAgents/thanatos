package testing

import "github.com/MythicMeta/MythicContainer/mythicrpc"

func (rpc *MockRPCExecutor) SendMythicRPCCallbackUpdate(
	input mythicrpc.MythicRPCCallbackUpdateMessage,
) (*mythicrpc.MythicRPCCallbackUpdateMessageResponse, error) {
	rpc.Callback = input

	return &mythicrpc.MythicRPCCallbackUpdateMessageResponse{
		Success: true,
	}, nil
}

func (rpc *MockRPCExecutor) SendMythicRPCResponseCreate(
	input mythicrpc.MythicRPCResponseCreateMessage,
) (*mythicrpc.MythicRPCResponseCreateMessageResponse, error) {
	rpc.Response = string(input.Response)

	return &mythicrpc.MythicRPCResponseCreateMessageResponse{
		Success: true,
	}, nil
}
