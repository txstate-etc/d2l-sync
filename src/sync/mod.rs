mod schemas;

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::URL_SAFE_NO_PAD;

use self::schemas::{UserReadOrUpdate, UserCreate, Activation};
pub use self::schemas::UserBase;
use hyper::StatusCode;
use reqwest::Client;
use std::io::Read;

const USR_PATH: &str = r#"/d2l/api/lp/1.20/users/"#;
const USR_QUERY: &str = r#"&userName="#;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct Sync {
    pub app_id: &'static str,
    pub app_key: &'static [u8],
    pub usr_id: &'static str,
    pub usr_key: &'static [u8],
    pub uri_base: &'static str,
    pub client: Client,
}

fn signature(key: &[u8], message: &[u8]) -> String {
    let mut mac = HmacSha256::new_varkey(key).unwrap();
    mac.input(message);
    base64::encode_config(&mac.result().code(), URL_SAFE_NO_PAD)
}

impl Sync {
    pub fn read(&self, user_base: &UserBase) -> Result<Option<usize>, FetchError> {
        let epoch = Utc::now().timestamp();
        let sig_body = format!("{}&{}&{}", "GET", USR_PATH, epoch);

        let app_sig = signature(self.app_key, sig_body.as_bytes());
        let usr_sig = signature(self.usr_key, sig_body.as_bytes());

        let uri = format!("{}{}?{}{}&x_a={}&x_c={}&x_b={}&x_d={}&x_t={}", self.uri_base, USR_PATH, USR_QUERY, user_base.user_name, self.app_id, app_sig, self.usr_id, usr_sig, epoch);
        let mut resp = self.client.get(&uri).send()?;
        if resp.status() == StatusCode::OK {
            let mut body = String::new();
            resp.read_to_string(&mut body)?;
            let user: UserReadOrUpdate = serde_json::from_str(&body)?;
            if user.user_base == *user_base && user.activation.is_active == true {
                Err(FetchError::NOP)
            } else {
                Ok(Some(user.user_id))
            }
        } else if resp.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Err(FetchError::StatusCode(resp.status()))
        }
    }

    pub fn update(&self, user_id: usize, user_base: &UserBase) -> Result<(), FetchError> {
        let user = UserReadOrUpdate {
            user_base: user_base.clone(),
            user_id: user_id,
            activation: Activation{is_active: true},
        };
        let epoch = Utc::now().timestamp();
        let sig_body = format!("{}&{}{}&{}", "PUT", USR_PATH, user_id, epoch);

        let app_sig = signature(self.app_key, sig_body.as_bytes());
        let usr_sig = signature(self.usr_key, sig_body.as_bytes());

        let uri = format!("{}{}{}?&x_a={}&x_c={}&x_b={}&x_d={}&x_t={}", self.uri_base, USR_PATH, user_id, self.app_id, app_sig, self.usr_id, usr_sig, epoch);
        let resp = self.client.put(&uri)
            .body(serde_json::to_string(&user)?)
            .send()?;
        if resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(FetchError::StatusCode(resp.status()))
        }
    }

    pub fn create(&self, role: String, user_base: &UserBase) -> Result<(), FetchError> {
        let user = UserCreate {
            user_base: user_base.clone(),
            role_id: role,
            is_active: true,
            send_creation_email: false,
        };

        let epoch = Utc::now().timestamp();
        let sig_body = format!("{}&{}&{}", "POST", USR_PATH, epoch);

        let app_sig = signature(self.app_key, sig_body.as_bytes());
        let usr_sig = signature(self.usr_key, sig_body.as_bytes());

        let uri = format!("{}{}?&x_a={}&x_c={}&x_b={}&x_d={}&x_t={}", self.uri_base, USR_PATH, self.app_id, app_sig, self.usr_id, usr_sig, epoch);
        let resp = self.client.post(&uri)
            .body(serde_json::to_string(&user)?)
            .send()?;
        if resp.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(FetchError::StatusCode(resp.status()))
        }
    }
}

#[derive(Debug)]
pub enum FetchError {
    NOP,
    Http(reqwest::Error),
    StatusCode(StatusCode),
    Json(serde_json::Error),
    IO(std::io::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> FetchError {
        FetchError::Http(err)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> FetchError {
        FetchError::Json(err)
    }
}

impl From<std::io::Error> for FetchError {
    fn from(err: std::io::Error) -> FetchError {
        FetchError::IO(err)
    }
}
