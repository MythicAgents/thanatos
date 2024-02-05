// Handles parsing the HTTP C2 profile parameters
package builder

import (
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"strconv"
	thanatoserror "thanatos/errors"
	"time"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyInfo<'a> {
    host: &'a str,
    port: u16,
    user: &'a str,
    pass: &'a str,
}
*/

// Data type with the HTTP proxy parameters
type HttpC2ProfileProxyParameters struct {
	Host string `msgpack:"host"`
	Port uint16 `msgpack:"port"`
	User string `msgpack:"user"`
	Pass string `msgpack:"pass"`
}

/*
{
  "crypto_type": {
    // TODO: Change type to an integer value rather than a string (serde_repr)
    "type": "aes256_hmac",
    "key": [1, 2, 3, 4, 5, ...]
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum CryptoInfo {
    #[serde(rename = "aes256_hmac")]
    Aes256Hmac {
        key: [u8; 16],
    }
}
*/

// HTTP C2 profile crypto information
type HttpC2ProfileCryptoInfo struct {
	Type string   `msgpack:"type"`
	Key  [16]byte `msgpack:"key"`
}

/*
/// HTTP profile configuration variables
#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfigVars<'a> {
    callback_host: &'a str,
    callback_interval: u32,
    callback_jitter: u16,
    callback_port: u16,
    killdate: u64,
    encrypted_exchange_check: bool,
    crypto_info: Option<CryptoInfo>,
    headers: HashMap<&'a str, &'a str>,
    get_uri: &'a str,
    post_uri: &'a str,
    query_path_name: &'a str,
    proxy_info: Option<ProxyInfo>,
}
*/

// Contains the HTTP C2 profile parameters
type HttpC2ProfileParameters struct {
	// Host for making HTTP connections to
	CallbackHost string `msgpack:"callback_host"`

	// Interval for HTTP connections
	CallbackInterval uint32 `msgpack:"callback_interval"`

	// Callback jitter for the payload
	CallbackJitter uint16 `msgpack:"callback_jitter"`

	// Callback port for the payload to connect to
	CallbackPort uint16 `msgpack:"callback_port"`

	// Killdate of the payload
	Killdate uint64 `msgpack:"killdate"`

	// Whether the payload should do a key exchange
	EncryptedExchangeCheck bool `msgpack:"encrypted_exchange_check"`

	// Information for encryption
	CryptoInfo *HttpC2ProfileCryptoInfo `msgpack:"crypto_info,omitempty"`

	// HTTP headers for making HTTP requests
	Headers map[string]string `msgpack:"headers"`

	// The GET uri for any GET requests
	GetUri string `msgpack:"get_uri"`

	// The POST uri for any POST requests
	PostUri string `msgpack:"post_uri"`

	// The query path for GET requests
	QueryPathName string `msgpack:"query_path_name"`

	// HTTP proxy information
	ProxyInfo *HttpC2ProfileProxyParameters `msgpack:"proxy_info,omitempty"`
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

	// Check if killdate timestamp integer conversion has overflowed. This shouldn't
	// happen until about another 292 billion years from now so I'll fix it then
	if killdateTime.Unix() > math.MaxInt64 {
		return &parsedParameters, thanatoserror.Errorf("are you from the future?")
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
