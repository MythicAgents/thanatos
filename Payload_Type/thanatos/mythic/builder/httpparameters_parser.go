// Handles parsing the HTTP C2 profile parameters
package builder

import (
	"errors"
	"strconv"
	builderrors "thanatos/builder/errors"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Data type with the HTTP proxy parameters
type HttpC2ProfileProxyParameters struct {
	Host string
	Port int
	User string
	Pass string
}

// Contains the parsed HTTP C2 profile parameters
type ParsedHttpC2ProfileParameters struct {
	// Callback port for the payload to connect to
	CallbackPort int64

	// Killdate of the payload
	Killdate time.Time

	// Whether the payload should do a key exchange
	EncryptedExchangeCheck bool

	// Callback jitter for the payload
	CallbackJitter int64

	// HTTP headers for making HTTP requests
	Headers map[string]string

	// Information for encryption
	CryptoInfo *struct {
		Type string
		Key  string
	}

	// Host for making HTTP connections to
	CallbackHost string

	// The GET uri for any GET requests
	GetUri string

	// The POST uri for any POST requests
	PostUri string

	// The query path for GET requests
	QueryPathName string

	// HTTP proxy information
	ProxyInfo *HttpC2ProfileProxyParameters

	// Interval for HTTP connections
	CallbackInterval int64
}

// Parses the HTTP C2 profile parameters
func parseHttpProfileParameters(parameters agentstructs.PayloadBuildC2Profile) (*ParsedHttpC2ProfileParameters, error) {
	const errorFormatStr string = "failed to get the '%s' value from the HTTP C2 profile parameters: %s"

	parsedParameters := ParsedHttpC2ProfileParameters{}

	callbackPort, err := parameters.GetNumberArg("callback_port")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "callback_port", err.Error())
	}

	if callbackPort < 1 || callbackPort > 65535 {
		return &parsedParameters, builderrors.New("configured callback port for the HTTP profile is not between 1 and 65535")
	}

	parsedParameters.CallbackPort = int64(callbackPort)

	killdate, err := parameters.GetDateArg("killdate")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "killdate", err.Error())
	}

	killdateTime, err := time.Parse(time.DateOnly, killdate)
	if err != nil {
		return &parsedParameters, builderrors.Errorf("failed to parse the HTTP profile killdate: %s", err.Error())
	}

	parsedParameters.Killdate = killdateTime

	encryptedExchangeCheck, err := parameters.GetBooleanArg("encrypted_exchange_check")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "encrypted_exchange_check", err.Error())
	}

	parsedParameters.EncryptedExchangeCheck = encryptedExchangeCheck

	callbackJitter, err := parameters.GetNumberArg("callback_jitter")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "callback_jitter", err.Error())
	}

	if callbackJitter < 0 || callbackJitter > 100 {
		return &parsedParameters, errors.New("callback jitter for the HTTP C2 profile is not between 0-99")
	}

	parsedParameters.CallbackJitter = int64(callbackJitter)

	headers, err := parameters.GetDictionaryArg("headers")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "headers", err.Error())
	}

	parsedParameters.Headers = headers

	aespsk, err := parameters.GetCryptoArg("AESPSK")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "AESPSK", err.Error())
	}

	if aespsk.Value != "none" {
		parsedParameters.CryptoInfo.Type = aespsk.Value
		parsedParameters.CryptoInfo.Key = aespsk.EncKey
	} else {
		parsedParameters.CryptoInfo = nil
	}

	callbackHost, err := parameters.GetStringArg("callback_host")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "callback_host", err.Error())
	}

	parsedParameters.CallbackHost = callbackHost

	getUri, err := parameters.GetStringArg("get_uri")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "get_uri", err.Error())
	}

	parsedParameters.GetUri = getUri

	postUri, err := parameters.GetStringArg("post_uri")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "post_uri", err.Error())
	}

	parsedParameters.PostUri = postUri

	queryPathName, err := parameters.GetStringArg("query_path_name")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "query_path_name", err.Error())
	}

	parsedParameters.QueryPathName = queryPathName

	proxyHost, err := parameters.GetStringArg("proxy_host")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "proxy_host", err.Error())
	}

	proxyPortStr, err := parameters.GetStringArg("proxy_port")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "proxy_port", err.Error())
	}

	proxyUser, err := parameters.GetStringArg("proxy_user")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "proxy_user", err.Error())
	}

	proxyPass, err := parameters.GetStringArg("proxy_pass")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "proxy_pass", err.Error())
	}

	// Proxy information is present
	if proxyHost != "" || proxyPortStr != "" || proxyUser != "" || proxyPass != "" {
		if proxyHost == "" {
			return &parsedParameters, builderrors.New("HTTP C2 profile proxy information supplied but proxy host is empty")
		}

		if proxyPortStr == "" {
			return &parsedParameters, builderrors.New("HTTP C2 profile proxy information supplied but proxy port is empty")
		}

		parsedParameters.ProxyInfo = &HttpC2ProfileProxyParameters{}
		parsedParameters.ProxyInfo.Host = proxyHost

		proxyPort, err := strconv.Atoi(proxyPortStr)
		if err != nil {
			return &parsedParameters, builderrors.Errorf("could not parse the 'proxy_port' value from the HTTP C2 profile parameters: %s", err.Error())
		}

		if proxyPort < 1 || proxyPort > 65535 {
			return &parsedParameters, builderrors.New("proxy port value for the HTTP C2 profile is outside the possible port range of 1-65535")
		}

		parsedParameters.ProxyInfo.Port = int(proxyPort)
		parsedParameters.ProxyInfo.User = proxyUser
		parsedParameters.ProxyInfo.Pass = proxyPass
	} else {
		parsedParameters.ProxyInfo = nil
	}

	callbackInterval, err := parameters.GetNumberArg("callback_interval")
	if err != nil {
		return &parsedParameters, builderrors.Errorf(errorFormatStr, "callback_interval", err.Error())
	}

	if callbackInterval < 0 {
		return &parsedParameters, builderrors.New("callback interval for the HTTP C2 profile is less than 0")
	}

	parsedParameters.CallbackInterval = int64(callbackInterval)
	return &parsedParameters, nil
}
