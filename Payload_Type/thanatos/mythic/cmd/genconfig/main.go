package main

import (
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"os"

	"github.com/MythicAgents/thanatos/pkg/builder"
	thanatoserror "github.com/MythicAgents/thanatos/pkg/errors"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"google.golang.org/protobuf/proto"
)

type payloadConfig struct {
	PayloadType     string                   `json:"payload_type"`
	PayloadUUID     string                   `json:"uuid"`
	C2Profiles      []payloadC2ProfileConfig `json:"c2_profiles"`
	BuildParameters []payloadBuildParameter  `json:"build_parameters"`
	Commands        []string                 `json:"commands"`
	SelectedOS      string                   `json:"selected_os"`
	Filename        string                   `json:"filename"`
	WrappedPayload  string                   `json:"wrapped_payload"`
}

type payloadC2ProfileConfig struct {
	Name       string                 `json:"c2_profile"`
	IsP2P      bool                   `json:"c2_profile_is_p2p"`
	Parameters map[string]interface{} `json:"c2_profile_parameters"`
}

type payloadBuildParameter struct {
	Name  string      `json:"name"`
	Value interface{} `json:"value"`
}

func main() {
	flagSet := flag.NewFlagSet("thanatos genconfig", flag.ExitOnError)

	inputFile := ""
	flagSet.Func("i", "Input Mythic payload JSON configuration file", func(filePath string) error {
		finfo, err := os.Stat(filePath)
		if err != nil {
			return err
		}

		if finfo.IsDir() {
			return errors.New("input path is a directory")
		}

		inputFile = filePath
		return nil
	})

	outputFile := ""
	flagSet.StringVar(&outputFile, "o", "", "Output path for the serialized configuration file")

	if len(os.Args) < 2 {
		flagSet.Usage()
		return
	}

	if err := flagSet.Parse(os.Args[1:]); err != nil {
		flagSet.Usage()
		return
	}

	if err := GenerateConfig(inputFile, outputFile); err != nil {
		fmt.Printf("failed to generate config:\n%s", err.Error())
		return
	}
}

func GenerateConfig(inputFile string, outputFile string) error {
	configBytes, err := os.ReadFile(inputFile)
	if err != nil {
		return thanatoserror.Errorf("failed to read input file: %s", err.Error())
	}

	var configData payloadConfig
	if err := json.Unmarshal(configBytes, &configData); err != nil {
		return thanatoserror.Errorf("failed to unmarshal config JSON: %s", err.Error())
	}

	paramMap := map[string]interface{}{}
	for _, param := range configData.BuildParameters {
		paramMap[param.Name] = param.Value
	}

	profiles := []agentstructs.PayloadBuildC2Profile{}
	for _, inputProfile := range configData.C2Profiles {
		profiles = append(profiles, agentstructs.PayloadBuildC2Profile{
			Name:       inputProfile.Name,
			IsP2P:      inputProfile.IsP2P,
			Parameters: inputProfile.Parameters,
		})
	}

	buildMsg := agentstructs.PayloadBuildMessage{
		PayloadType: configData.PayloadType,
		Filename:    configData.Filename,
		CommandList: configData.Commands,
		BuildParameters: agentstructs.PayloadBuildArguments{
			Parameters: paramMap,
		},
		C2Profiles:         profiles,
		WrappedPayload:     nil,
		WrappedPayloadUUID: nil,
		SelectedOS:         configData.SelectedOS,
		PayloadUUID:        configData.PayloadUUID,
	}

	outputConfig, err := builder.CreateConfig(buildMsg)
	if err != nil {
		return errors.Join(thanatoserror.New("failed to create config"), err)
	}

	serialized, err := proto.Marshal(outputConfig)
	if err != nil {
		return thanatoserror.Errorf("could not marshal config: %s", err.Error())
	}

	if err := os.WriteFile(outputFile, serialized, 0644); err != nil {
		return thanatoserror.Errorf("failed to write output config: %s", err.Error())
	}

	return nil
}
