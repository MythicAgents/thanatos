// Handles parsing the HTTP C2 profile parameters
package builder

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"time"

	thanatoserror "github.com/MythicAgents/thanatos/errors"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

// Data type with the HTTP proxy parameters
type HttpC2ProfileProxyParameters struct {
	Host string
	Port uint16
	User string
	Pass string
}

// HTTP C2 profile crypto information
type HttpC2ProfileCryptoInfo struct {
	Type string
	Key  [16]byte
}

// Contains the HTTP C2 profile parameters
type HttpC2ProfileParameters struct {
	// Host for making HTTP connections to
	CallbackHost string

	// Interval for HTTP connections
	CallbackInterval uint32

	// Callback jitter for the payload
	CallbackJitter uint16

	// Callback port for the payload to connect to
	CallbackPort uint16

	// Killdate of the payload
	Killdate uint64

	// Whether the payload should do a key exchange
	EncryptedExchangeCheck bool

	// Information for encryption
	CryptoInfo *HttpC2ProfileCryptoInfo

	// HTTP headers for making HTTP requests
	Headers map[string]string

	// The GET uri for any GET requests
	GetUri string

	// The POST uri for any POST requests
	PostUri string

	// The query path for GET requests
	QueryPathName string

	// HTTP proxy information
	ProxyInfo *HttpC2ProfileProxyParameters
}

func (p *HttpC2ProfileParameters) String() string {
	output := ""

	outputFormat := "HTTP_CALLBACK_HOST=%s\n" +
		"HTTP_CALLBACK_INTERVAL=%d\n" +
		"HTTP_CALLBACK_JITTER=%d\n" +
		"HTTP_CALLBACK_PORT=%d\n" +
		"HTTP_KILLDATE=%d\n" +
		"HTTP_ENCRYPTED_EXCHANGE_CHECK=%t\n" +
		"HTTP_HEADERS=%s\n" +
		"HTTP_GET_URI=%s\n" +
		"HTTP_POST_URI=%s\n" +
		"HTTP_QUERY_PATH_NAME=%s\n"

	headers, _ := json.Marshal(&p.Headers)
	headers_encoded := base64.StdEncoding.EncodeToString(headers)

	output += fmt.Sprintf(outputFormat,
		p.CallbackHost,
		p.CallbackInterval,
		p.CallbackJitter,
		p.CallbackPort,
		p.Killdate,
		p.EncryptedExchangeCheck,
		headers_encoded,
		p.GetUri,
		p.PostUri,
		p.QueryPathName,
	)

	if p.CryptoInfo != nil {
		output += fmt.Sprintf("HTTP_CRYPTO_TYPE=%s\n", p.CryptoInfo.Type)
		key := base64.StdEncoding.EncodeToString(p.CryptoInfo.Key[:])
		output += fmt.Sprintf("HTTP_CRYPTO_KEY=%s\n", key)
	}

	if p.ProxyInfo != nil {
		outputFormat = "HTTP_PROXY_HOST=%s\n" +
			"HTTP_PROXY_PORT=%d\n" +
			"HTTP_PROXY_USER=%s\n" +
			"HTTP_PROXY_PASS=%s\n"

		output += fmt.Sprintf(outputFormat, p.ProxyInfo.Host, p.ProxyInfo.Port, p.ProxyInfo.User, p.ProxyInfo.Pass)
	}

	return output
}

// Parses the HTTP C2 profile parameters
func parseHttpProfileParameters(parameters agentstructs.PayloadBuildC2Profile) (*HttpC2ProfileParameters, error) {
	const errorFormatStr string = "failed to get the '%s' value from the HTTP C2 profile parameters: %s"

	parsedParameters := HttpC2ProfileParameters{}

	callbackPort, err := parameters.GetNumberArg("callback_port")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "callback_port", err.Error())
	}

	if callbackPort < 1 || callbackPort > 65535 {
		return &parsedParameters, thanatoserror.New("configured callback port for the HTTP profile is not between 1 and 65535")
	}

	parsedParameters.CallbackPort = uint16(callbackPort)

	killdate, err := parameters.GetDateArg("killdate")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "killdate", err.Error())
	}

	killdateTime, err := time.Parse(time.DateOnly, killdate)
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf("failed to parse the HTTP profile killdate: %s", err.Error())
	}

	parsedParameters.Killdate = uint64(killdateTime.Unix())

	encryptedExchangeCheck, err := parameters.GetBooleanArg("encrypted_exchange_check")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "encrypted_exchange_check", err.Error())
	}

	parsedParameters.EncryptedExchangeCheck = encryptedExchangeCheck

	callbackJitter, err := parameters.GetNumberArg("callback_jitter")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "callback_jitter", err.Error())
	}

	if callbackJitter < 0 || callbackJitter > 100 {
		return &parsedParameters, errors.New("callback jitter for the HTTP C2 profile is not between 0-99")
	}

	parsedParameters.CallbackJitter = uint16(callbackJitter)

	headers, err := parameters.GetDictionaryArg("headers")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "headers", err.Error())
	}

	parsedParameters.Headers = headers

	aespsk, err := parameters.GetCryptoArg("AESPSK")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "AESPSK", err.Error())
	}

	if aespsk.Value != "none" {
		aeskey, err := base64.StdEncoding.DecodeString(aespsk.EncKey)
		if err != nil {
			return &parsedParameters, thanatoserror.Errorf("failed to base64 decode HTTP encryption key: %s", err.Error())
		}

		parsedParameters.CryptoInfo.Type = aespsk.Value
		parsedParameters.CryptoInfo.Key = [16]byte(aeskey)
	} else {
		parsedParameters.CryptoInfo = nil
	}

	callbackHost, err := parameters.GetStringArg("callback_host")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "callback_host", err.Error())
	}

	parsedParameters.CallbackHost = callbackHost

	getUri, err := parameters.GetStringArg("get_uri")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "get_uri", err.Error())
	}

	parsedParameters.GetUri = getUri

	postUri, err := parameters.GetStringArg("post_uri")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "post_uri", err.Error())
	}

	parsedParameters.PostUri = postUri

	queryPathName, err := parameters.GetStringArg("query_path_name")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "query_path_name", err.Error())
	}

	parsedParameters.QueryPathName = queryPathName

	proxyHost, err := parameters.GetStringArg("proxy_host")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "proxy_host", err.Error())
	}

	proxyPortStr, err := parameters.GetStringArg("proxy_port")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "proxy_port", err.Error())
	}

	proxyUser, err := parameters.GetStringArg("proxy_user")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "proxy_user", err.Error())
	}

	proxyPass, err := parameters.GetStringArg("proxy_pass")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "proxy_pass", err.Error())
	}

	// Proxy information is present
	if proxyHost != "" || proxyPortStr != "" || proxyUser != "" || proxyPass != "" {
		if proxyHost == "" {
			return &parsedParameters, thanatoserror.New("HTTP C2 profile proxy information supplied but proxy host is empty")
		}

		if proxyPortStr == "" {
			return &parsedParameters, thanatoserror.New("HTTP C2 profile proxy information supplied but proxy port is empty")
		}

		parsedParameters.ProxyInfo = &HttpC2ProfileProxyParameters{}
		parsedParameters.ProxyInfo.Host = proxyHost

		proxyPort, err := strconv.Atoi(proxyPortStr)
		if err != nil {
			return &parsedParameters, thanatoserror.Errorf("could not parse the 'proxy_port' value from the HTTP C2 profile parameters: %s", err.Error())
		}

		if proxyPort < 1 || proxyPort > 65535 {
			return &parsedParameters, thanatoserror.New("proxy port value for the HTTP C2 profile is outside the possible port range of 1-65535")
		}

		parsedParameters.ProxyInfo.Port = uint16(proxyPort)
		parsedParameters.ProxyInfo.User = proxyUser
		parsedParameters.ProxyInfo.Pass = proxyPass
	} else {
		parsedParameters.ProxyInfo = nil
	}

	callbackInterval, err := parameters.GetNumberArg("callback_interval")
	if err != nil {
		return &parsedParameters, thanatoserror.Errorf(errorFormatStr, "callback_interval", err.Error())
	}

	if callbackInterval < 0 {
		return &parsedParameters, thanatoserror.New("callback interval for the HTTP C2 profile is less than 0")
	}

	parsedParameters.CallbackInterval = uint32(callbackInterval)
	return &parsedParameters, nil
}
