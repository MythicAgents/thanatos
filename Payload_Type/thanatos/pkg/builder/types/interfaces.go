package types

import (
	"github.com/MythicMeta/MythicContainer/mythicrpc"
)

// Generic handler interface for managing payload builds and RPC execution
type BuildHandler interface {
	PayloadBuilder
	MythicRPCExecutor
}

// Interface handling various payload build routines
type PayloadBuilder interface {
	// Method which takes in the raw command for building the agent and returns the contents
	// of the built payload for Mythic
	RunBuildCommand(command string, target string, output PayloadBuildParameterOutputFormat) ([]byte, error)
}

// Interface for execution Mythic RPC routines
type MythicRPCExecutor interface {
	// Updates the build step in Mythic
	UpdateBuildStep(input mythicrpc.MythicRPCPayloadUpdateBuildStepMessage) (*mythicrpc.MythicRPCPayloadUpdateBuildStepMessageResponse, error)
}
