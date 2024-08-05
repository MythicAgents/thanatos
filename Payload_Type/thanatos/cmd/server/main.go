package main

import (
	"github.com/MythicAgents/thanatos/pkg/builder"
	"github.com/MythicAgents/thanatos/pkg/commands"
	"github.com/MythicMeta/MythicContainer"
)

func main() {
	// Initialize the builder
	builder.Initialize()

	// Initialize the commands
	commands.Initialize()

	// Start communicating with Mythic
	MythicContainer.StartAndRunForever([]MythicContainer.MythicServices{
		MythicContainer.MythicServicePayload,
	})
}
