package builder

import (
	"crypto/sha256"

	"github.com/MythicAgents/thanatos/pb/config"
)

func createConfig(payloadParameters ParsedPayloadParameters) *config.Config {
	payloadConfig := config.Config{}

	payloadConfig.Uuid = payloadParameters.Uuid[:]

	payloadConfig.WorkingHoursStart = uint32(payloadParameters.BuildParameters.WorkingHours.StartTime.Seconds())
	payloadConfig.WorkingHoursEnd = uint32(payloadParameters.BuildParameters.WorkingHours.EndTime.Seconds())

	payloadConfig.ConnectionRetries = payloadParameters.BuildParameters.ConnectionRetries
	payloadConfig.SpawnTo = payloadParameters.BuildParameters.SpawnTo

	if len(payloadParameters.BuildParameters.DomainList) > 0 {
		hashedDomains := make([]byte, len(payloadParameters.BuildParameters.DomainList)*32)
		for _, domain := range payloadParameters.BuildParameters.DomainList {
			h := sha256.New()
			h.Write([]byte(domain))
			hashedDomains = append(hashedDomains, h.Sum(nil)...)
		}

		payloadConfig.Domains = hashedDomains
	}

	if len(payloadParameters.BuildParameters.HostnameList) > 0 {
		hashedHostnames := make([]byte, len(payloadParameters.BuildParameters.HostnameList)*32)
		for _, domain := range payloadParameters.BuildParameters.HostnameList {
			h := sha256.New()
			h.Write([]byte(domain))
			hashedHostnames = append(hashedHostnames, h.Sum(nil)...)
		}

		payloadConfig.Hostnames = hashedHostnames
	}

	if len(payloadParameters.BuildParameters.UsernameList) > 0 {
		hashedUsernames := make([]byte, len(payloadParameters.BuildParameters.UsernameList)*32)
		for _, domain := range payloadParameters.BuildParameters.UsernameList {
			h := sha256.New()
			h.Write([]byte(domain))
			hashedUsernames = append(hashedUsernames, h.Sum(nil)...)
		}

		payloadConfig.Usernames = hashedUsernames
	}

	if payloadParameters.C2Profiles.HttpC2Profile != nil {
		payloadConfig.Http = &config.HttpConfig{
			CallbackPort:     uint32(payloadParameters.C2Profiles.HttpC2Profile.CallbackPort),
			Killdate:         payloadParameters.C2Profiles.HttpC2Profile.Killdate,
			CallbackJitter:   uint32(payloadParameters.C2Profiles.HttpC2Profile.CallbackJitter),
			Headers:          payloadParameters.C2Profiles.HttpC2Profile.Headers,
			CallbackHost:     payloadParameters.C2Profiles.HttpC2Profile.CallbackHost,
			GetUri:           payloadParameters.C2Profiles.HttpC2Profile.GetUri,
			PostUri:          payloadParameters.C2Profiles.HttpC2Profile.PostUri,
			QueryPathName:    payloadParameters.C2Profiles.HttpC2Profile.QueryPathName,
			CallbackInterval: uint32(payloadParameters.C2Profiles.HttpC2Profile.CallbackInterval),
		}

		if payloadParameters.C2Profiles.HttpC2Profile.CryptoInfo != nil {
			payloadConfig.Http.AesKey = payloadParameters.C2Profiles.HttpC2Profile.CryptoInfo.Key[:]
		}

		if payloadParameters.C2Profiles.HttpC2Profile.ProxyInfo != nil {
			payloadConfig.Http.Proxy = &config.ProxyInfo{
				Host: payloadParameters.C2Profiles.HttpC2Profile.ProxyInfo.Host,
				Port: uint32(payloadParameters.C2Profiles.HttpC2Profile.ProxyInfo.Port),
				Pass: payloadParameters.C2Profiles.HttpC2Profile.ProxyInfo.Pass,
			}
		}
	}

	return &payloadConfig
}
