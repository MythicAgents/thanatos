package commands

import (
	"github.com/MythicAgents/thanatos/pkg/commands/exit"
	"github.com/MythicAgents/thanatos/pkg/commands/sleep"
)

func Initialize() {
	exit.Initialize()
	sleep.Initialize()
}
