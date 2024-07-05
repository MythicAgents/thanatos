package commands

import (
	"github.com/MythicAgents/thanatos/commands/exit"
	"github.com/MythicAgents/thanatos/commands/sleep"
)

func Initialize() {
	exit.Initialize()
	sleep.Initialize()
}
