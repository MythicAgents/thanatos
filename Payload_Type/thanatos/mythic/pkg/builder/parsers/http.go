package parsers

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

var parseMap = map[string]func(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error{
	"callback_port":            parseCallbackPort,
	"killdate":                 parseKilldate,
	"encrypted_exchange_check": parseEncryptedExchangeCheck,
	"callback_jitter":          parseCallbackJitter,
	"headers":                  parseHeaders,
	"AESPSK":                   parseAesPsk,
	"callback_host":            parseCallbackHost,
	"get_uri":                  parseGetUri,
	"post_uri":                 parsePostUri,
	"query_path_name":          parseQueryPathName,
	"proxy_host":               parseProxyHost,
	"proxy_port":               parseProxyPort,
	"proxy_user":               parseProxyUser,
	"proxy_pass":               parseProxyPass,
	"callback_interval":        parseCallbackInterval,
}

func ParseHttpProfile(resultConfig *config.Config, profile agentstructs.PayloadBuildC2Profile) error {
	resultConfig.Http = &config.HttpConfig{}

	for param := range profile.Parameters {
		if parseFn, ok := parseMap[param]; ok {
			if err := parseFn(param, resultConfig, &profile); err != nil {
				return errors.Join(thanatoserror.Errorf("failed to parse %s http profile parameter", param), err)
			}
		}
	}

	return nil
}

func parseCallbackPort(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	port, err := profile.GetNumberArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if port > 65535 || port <= 0 {
		return thanatoserror.New("callback port is invalid")
	}

	c.Http.CallbackPort = uint32(port)
	return nil
}

func parseKilldate(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	killdate, err := profile.GetDateArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get paramter: %s", err.Error())
	}

	d, err := time.Parse(time.DateOnly, killdate)
	if err != nil {
		return thanatoserror.Errorf("failed to parse killdate: %s", err.Error())
	}

	c.Http.Killdate = uint64(d.Unix())
	return nil
}

func parseEncryptedExchangeCheck(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	eke, err := profile.GetBooleanArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.Eke = eke
	return nil
}

func parseCallbackJitter(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	jitter, err := profile.GetNumberArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if jitter < 0 || jitter >= 100 {
		return thanatoserror.New("callback jitter is not between 0-99")
	}

	c.Http.CallbackJitter = uint32(jitter)
	return nil
}

func parseHeaders(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	headers, err := profile.GetDictionaryArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.Headers = headers
	return nil
}

func parseAesPsk(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
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

		c.Http.AesKey = aesKey
	}

	return nil
}

func parseCallbackHost(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	callback_host, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.CallbackHost = callback_host
	return nil
}

func parseGetUri(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	getUri, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.GetUri = getUri
	return nil
}

func parsePostUri(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	postUri, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.PostUri = postUri
	return nil
}

func parseQueryPathName(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	queryPath, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	c.Http.QueryPathName = queryPath
	return nil
}

func parseProxyHost(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	proxyHost, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if c.Http.Proxy == nil {
		c.Http.Proxy = &config.ProxyInfo{}
	}

	c.Http.Proxy.Host = proxyHost
	return nil
}

func parseProxyPort(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	proxyPort, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
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

	if c.Http.Proxy == nil {
		c.Http.Proxy = &config.ProxyInfo{}
	}

	c.Http.Proxy.Port = uint32(portValue)
	return nil
}

func parseProxyUser(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	proxyUser, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if c.Http.Proxy == nil {
		c.Http.Proxy = &config.ProxyInfo{}
	}

	c.Http.Proxy.User = proxyUser
	return nil
}

func parseProxyPass(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	proxyPass, err := profile.GetStringArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if c.Http.Proxy == nil {
		c.Http.Proxy = &config.ProxyInfo{}
	}
	c.Http.Proxy.Pass = proxyPass
	return nil

}

func parseCallbackInterval(name string, c *config.Config, profile *agentstructs.PayloadBuildC2Profile) error {
	interval, err := profile.GetNumberArg(name)
	if err != nil {
		return thanatoserror.Errorf("could not get parameter: %s", err.Error())
	}

	if interval < 0 || interval > math.MaxUint32 {
		return thanatoserror.New("callback interval is not valid")
	}

	c.Http.CallbackInterval = uint32(interval)
	return nil
}
