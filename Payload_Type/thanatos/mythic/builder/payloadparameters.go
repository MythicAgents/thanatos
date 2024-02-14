package builder

import (
	"slices"
	thanatoserror "thanatos/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/uuid"
)

type ParsedPayloadParameters struct {
	Uuid            uuid.UUID
	SelectedOS      string
	Filename        string
	Commands        []string
	BuildParameters ParsedBuildParameters
	IsP2P           bool
	C2Profiles      struct {
		HttpC2Profile *HttpC2ProfileParameters
		TcpC2Profile  *string
	}
}

func parsePayloadParameters(buildMessage agentstructs.PayloadBuildMessage) (ParsedPayloadParameters, error) {
	payloadUUID, err := uuid.Parse(buildMessage.PayloadUUID)
	if err != nil {
		return ParsedPayloadParameters{}, thanatoserror.Errorf("failed to parse payload UUID: %s", err.Error())
	}

	buildParameters, err := parsePayloadBuildParameters(buildMessage)
	if err != nil {
		return ParsedPayloadParameters{}, thanatoserror.Errorf("failed to parse build parameters: %s", err.Error())
	}

	commands := slices.DeleteFunc(buildMessage.CommandList, func(s string) bool {
		return slices.Contains(builtinCommands, s)
	})

	parameters := ParsedPayloadParameters{
		Uuid:            payloadUUID,
		SelectedOS:      buildMessage.SelectedOS,
		Filename:        buildMessage.Filename,
		Commands:        commands,
		BuildParameters: buildParameters,
	}

	for _, profile := range buildMessage.C2Profiles {
		switch profile.Name {
		case "http":
			httpProfile, err := parseHttpProfileParameters(profile)
			if err != nil {
				return ParsedPayloadParameters{}, thanatoserror.Errorf("failed to parse HTTP C2 profile parameters: %s", err.Error())
			}
			parameters.C2Profiles.HttpC2Profile = httpProfile
		}
	}

	if parameters.C2Profiles.HttpC2Profile != nil && parameters.C2Profiles.TcpC2Profile != nil {
		return parameters, thanatoserror.New("cannot mix egress and p2p C2 profiles")
	}

	if parameters.C2Profiles.HttpC2Profile != nil {
		parameters.IsP2P = false
	} else {
		parameters.IsP2P = true
	}

	return parameters, nil
}
