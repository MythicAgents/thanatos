package builder

import (
	"errors"

	"github.com/MythicAgents/thanatos/builder/parsers"
	thanatoserror "github.com/MythicAgents/thanatos/errors"
	pbconfig "github.com/MythicAgents/thanatos/pb/config"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
	"github.com/google/uuid"
	"golang.org/x/exp/slices"
)

func CreateConfig(buildMsg agentstructs.PayloadBuildMessage) (*pbconfig.Config, error) {
	payloadUuid, err := uuid.Parse(buildMsg.PayloadUUID)
	if err != nil {
		return nil, thanatoserror.Errorf("failed to parse payload uuid: %s", err.Error())
	}

	resultConfig := &pbconfig.Config{
		Uuid: payloadUuid[:],
	}

	if err := parsePayloadParameters(resultConfig, buildMsg); err != nil {
		return nil, errors.Join(thanatoserror.New("failed to parse payload build parameters"), err)
	}

	egressEnabled := false
	p2pEnabled := false

	for _, profile := range buildMsg.C2Profiles {
		switch profile.Name {
		case "http":
			egressEnabled = true
			if err := parsers.ParseHttpProfile(resultConfig, profile); err != nil {
				return nil, errors.Join(thanatoserror.New("failed to parse http profile parameters"), err)
			}
		case "tcp":
			p2pEnabled = true
			return nil, thanatoserror.New("tcp profile unimplemented")
		}
	}

	if egressEnabled && p2pEnabled {
		return nil, thanatoserror.Errorf("cannot mix egress and p2p C2 profiles")
	}

	return resultConfig, nil
}

func parsePayloadParameters(resultConfig *pbconfig.Config, buildMsg agentstructs.PayloadBuildMessage) error {
	for name := range buildMsg.BuildParameters.Parameters {
		paramIndex := slices.IndexFunc(ThanatosBuildParameters, func(param ThanatosBuildParameter) bool {
			return param.Name == name
		})

		if paramIndex == -1 {
			return thanatoserror.Errorf("failed to find build parameter %s", name)
		}

		if parseFn := ThanatosBuildParameters[paramIndex].ParseFunction; parseFn != nil {
			if err := parseFn(name, resultConfig, &buildMsg); err != nil {
				return errors.Join(thanatoserror.Errorf("failed to parse %s build parameter", name), err)
			}
		}
	}
	return nil
}
