package builder

import (
	"encoding/base64"
	"errors"
	"math"
	"strconv"
	"time"

	thanatoserror "github.com/MythicAgents/thanatos/pkg/errors"
	"github.com/MythicAgents/thanatos/proto/config"
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

var httpParameterMap = map[string]func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error{
	"callback_port": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		port, err := profile.GetNumberArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if port > 65535 || port <= 0 {
			return thanatoserror.New("callback port is invalid")
		}

		c.CallbackPort = uint32(port)
		return nil
	},
	"killdate": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		killdate, err := profile.GetDateArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get paramter: %s", err.Error())
		}

		d, err := time.Parse(time.DateOnly, killdate)
		if err != nil {
			return thanatoserror.Errorf("failed to parse killdate: %s", err.Error())
		}

		c.Killdate = uint64(d.Unix())
		return nil

	},
	"encrypted_exchange_check": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		eke, err := profile.GetBooleanArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.Eke = eke
		return nil
	},
	"callback_jitter": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		jitter, err := profile.GetNumberArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if jitter < 0 || jitter >= 100 {
			return thanatoserror.New("callback jitter is not between 0-99")
		}

		c.CallbackJitter = uint32(jitter)
		return nil
	},
	"headers": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		headers, err := profile.GetDictionaryArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.Headers = headers
		return nil
	},
	"AESPSK": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		aespsk, err := profile.GetCryptoArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if aespsk.Value == "aes256_hmac" {
			aesKey, err := base64.StdEncoding.DecodeString(aespsk.EncKey)
			if err != nil {
				return thanatoserror.Errorf("failed to base64 decode AES key: %s", err.Error())
			}

			if len(aesKey) > 32 {
				return thanatoserror.Errorf("AES key is an invalid size of %d", len(aesKey))
			}

			c.AesKey = aesKey
		}

		return nil
	},
	"callback_host": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		callback_host, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.CallbackHost = callback_host
		return nil
	},
	"get_uri": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		getUri, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.GetUri = getUri
		return nil
	},
	"post_uri": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		postUri, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.PostUri = postUri
		return nil
	},
	"query_path_name": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		queryPath, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		c.QueryPathName = queryPath
		return nil
	},
	"proxy_host": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		proxyHost, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if len(proxyHost) == 0 {
			return nil
		}

		if c.Proxy == nil {
			c.Proxy = &config.ProxyInfo{}
		}

		c.Proxy.Host = proxyHost
		return nil
	},
	"proxy_port": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		proxyPort, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if len(proxyPort) == 0 {
			return nil
		}

		if proxyPort == "" {
			return nil
		}

		portValue, err := strconv.Atoi(proxyPort)
		if err != nil {
			return thanatoserror.New("port value is not an integer")
		}

		if portValue <= 0 || portValue > 65535 {
			return thanatoserror.New("proxy port is not valid")
		}

		if c.Proxy == nil {
			c.Proxy = &config.ProxyInfo{}
		}

		c.Proxy.Port = uint32(portValue)
		return nil
	},
	"proxy_user": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		proxyUser, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if len(proxyUser) == 0 {
			return nil
		}

		if c.Proxy == nil {
			c.Proxy = &config.ProxyInfo{}
		}

		c.Proxy.User = proxyUser
		return nil
	},
	"proxy_pass": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		proxyPass, err := profile.GetStringArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if len(proxyPass) == 0 {
			return nil
		}

		if c.Proxy == nil {
			c.Proxy = &config.ProxyInfo{}
		}
		c.Proxy.Pass = proxyPass
		return nil
	},
	"callback_interval": func(name string, c *config.HttpConfig, profile *agentstructs.PayloadBuildC2Profile) error {
		interval, err := profile.GetNumberArg(name)
		if err != nil {
			return thanatoserror.Errorf("could not get parameter: %s", err.Error())
		}

		if interval < 0 || interval > math.MaxUint32 {
			return thanatoserror.New("callback interval is not valid")
		}

		c.CallbackInterval = uint32(interval)
		return nil
	},
}

func ParseHttpProfile(resultConfig *config.Config, profile agentstructs.PayloadBuildC2Profile) error {
	httpConfig := config.HttpConfig{}

	for param := range profile.Parameters {
		if parseFn, ok := httpParameterMap[param]; ok {
			if err := parseFn(param, &httpConfig, &profile); err != nil {
				return errors.Join(thanatoserror.Errorf("failed to parse %s http profile parameter", param), err)
			}
		}
	}

	resultConfig.Profile = &config.Config_Http{
		Http: &httpConfig,
	}
	return nil
}
