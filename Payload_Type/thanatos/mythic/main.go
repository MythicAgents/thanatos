// This is the entry point for the server side portion of the payload type.
// The code in this module is in charge of dispatching and receiving tasking and building
// new payloads.
package main

import (
	"errors"
	"flag"
	"fmt"
	"os"

	"github.com/MythicAgents/thanatos/builder"
	"github.com/MythicAgents/thanatos/commands"
	"github.com/MythicMeta/MythicContainer"
)

func main() {
	flagSet := flag.NewFlagSet("thanatos", flag.ContinueOnError)

	helpFlag := false
	flagSet.BoolVar(&helpFlag, "h", false, "Print this help menu")
	flagSet.Usage = thanatosUsage

	if len(os.Args) < 2 {
		flagSet.Usage()
		os.Exit(0)
	}

	flagSet.Parse(os.Args[1:])

	if helpFlag {
		flagSet.Usage()
		os.Exit(0)
	}

	args := flagSet.Args()
	if len(args) == 0 {
		flagSet.Usage()
		os.Exit(0)
	}

	switch args[0] {
	case "server":
		server(args[1:])
		return
	case "genconfig":
		genconfig(args[1:])
		return
	}

	flagSet.Usage()
}

func server(args []string) {
	flagSet := flag.NewFlagSet("thanatos server", flag.ContinueOnError)

	helpFlag := false
	flagSet.BoolVar(&helpFlag, "h", false, "Print this help menu")
	flagSet.Parse(args)

	if helpFlag {
		flagSet.Usage()
		os.Exit(0)
	}

	runServer()
}

func runServer() {
	// Initialize the builder
	builder.Initialize()

	// Initialize the commands
	commands.Initialize()

	// Start communicating with Mythic
	MythicContainer.StartAndRunForever([]MythicContainer.MythicServices{
		MythicContainer.MythicServicePayload,
	})
}

func genconfig(args []string) {
	flagSet := flag.NewFlagSet("thanatos genconfig", flag.ExitOnError)

	inputFile := ""
	flagSet.Func("i", "Input Mythic payload JSON configuration file", func(filePath string) error {
		finfo, err := os.Stat(filePath)
		if err != nil {
			return err
		}

		if finfo.IsDir() {
			return errors.New("Input path is a directory")
		}

		inputFile = filePath
		return nil
	})

	outputFile := ""
	flagSet.StringVar(&outputFile, "o", "", "Output path for the serialized configuration file")
	if err := flagSet.Parse(args); err != nil {
		flagSet.Usage()
		os.Exit(0)
	}

	if err := GenerateConfig(inputFile, outputFile); err != nil {
		fmt.Printf("failed to generate config:\n%s\n", err.Error())
		os.Exit(1)
	}
}

func thanatosUsage() {
	var usageString = `Usage of thanatos:
  -h         Print this help menu
  server     Run the thanatos server
  genconfig  Generate a serialized configuration from a Mythic JSON config
`
	fmt.Printf("%s", usageString)
}
