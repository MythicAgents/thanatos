// This is the entry point for the server side portion of the payload type.
// The code in this module is in charge of dispatching and receiving tasking and building
// new payloads.
package main

import (
	"thanatos/builder"
	//command "thanatos/command"
	//"github.com/MythicMeta/MythicContainer"
)

func main() {
	// Initialize the builder
	builder.Initialize()

	// Initialize the commands
	//command.Initialize()

	// Start communicating with Mythic
	//MythicContainer.StartAndRunForever([]MythicContainer.MythicServices{
	//	MythicContainer.MythicServicePayload,
	//})
}
