#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate hmac;
extern crate sha2;
extern crate base64;
extern crate hyper;
extern crate reqwest;

mod sync;

//use chrono::Utc;
use std::time::Duration;
use std::env;

use sync::{Sync, UserBase, FetchError};
use reqwest::Client;

lazy_static! {
    static ref APP_ID: String = {
        env::var("D2L_APP_ID").expect("D2L_APP_ID required")
    };
}

lazy_static! {
    static ref APP_KEY: Vec<u8> = {
        env::var("D2L_APP_KEY").expect("D2L_APP_KEY required").into_bytes()
    };
}

lazy_static! {
    static ref USR_ID: String = {
        env::var("D2L_USR_ID").expect("D2L_USR_ID required")
    };
}

lazy_static! {
    static ref USR_KEY: Vec<u8> = {
        env::var("D2L_USR_KEY").expect("D2L_USR_KEY required").into_bytes()
    };
}

lazy_static! {
    static ref URI_BASE: String = {
        match env::var("D2L_URI_BASE") {
            Ok(uri_base) => uri_base,
            Err(_) => "https://test.brightspace.com".to_string(),
        }
    };
}

fn main() {
    let user = UserBase {
        first_name: "Nathan".to_string(),
        middle_name: "W".to_string(),
        last_name: "Pino".to_string(),
        user_name: "np11@txstate.edu".to_string(),
        org_defined_id: Some("A00181786".to_string()),
        external_email: Some("np11@txstate.edu".to_string()),
    };
    let client = Client::builder()
        .timeout(Duration::from_secs(360))
        .build().expect("Unable to create client");
    let sync = Sync {
        app_id: &*APP_ID,
        app_key: &*APP_KEY,
        usr_id: &*USR_ID,
        usr_key: &*USR_KEY,
        uri_base: &*URI_BASE,
        client: client,
    };
    match sync.read(&user) {
        Ok(Some(user_id)) => match sync.update(user_id, &user) {
            Ok(()) => println!("Updated user: {:?}", user.user_name),
            Err(e) => eprintln!("Error while updating user {:?}, {:?}", user, e),
        },
        Ok(None) => match sync.create("109".to_string(), &user) {
            Ok(()) => println!("Created user: {:?}", user.user_name),
            Err(e) => eprintln!("Error while creating user {:?}, {:?}", user, e),
        },
        Err(FetchError::NOP) => println!("No update required for {}", user.user_name),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
