package commands

import (
	"thanatos/commands/exit"
	"thanatos/commands/sleep"
)

func Initialize() {
	exit.Initialize()
	sleep.Initialize()
}
