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
extern crate mysql;

mod sync;
mod source;

//use chrono::Utc;
use std::time::Duration;
use std::env;
use std::str::FromStr;

use source::Source;
use sync::{Sync, UserBase, Role};
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

/// SOURCE environment variable contains
/// the "warehouse" connection uri.
/// This database will be used to read
/// journal entries to find users that
/// need to be updated as well as pull
/// user information used to sync d2l
/// Example: "mysql://usr:pwd@host:port/database?options"
lazy_static! {
    static ref SOURCE: Option<String> = {
        match env::var("D2L_SOURCE") {
            Ok(uri) => Some(uri),
            Err(_) => None,
        }
    };
}

lazy_static! {
    static ref QUERY_JOURNAL: Option<String> = {
        match env::var("D2L_QUERY_JOURNAL") {
            Ok(q) => Some(q),
            Err(_) => None,
        }
    };
}

lazy_static! {
    static ref QUERY_USER: Option<String> = {
        match env::var("D2L_QUERY_USER") {
            Ok(q) => Some(q),
            Err(_) => None,
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut args = args.iter();
    let mut id: Option<usize> = None;
    let mut data: Option<UserBase> = None;
    let mut role: Option<Role> = Some(Role::Student);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--data" => if let Some(d) = args.next() {
                data = Some(serde_json::from_str(&d).unwrap());
            },
            "-r" | "--role"  => if let Some(r) = args.next() {
                role = Some(Role::from_str(&r).unwrap());
            },
            "-i" | "--id" => if let Some(i) = args.next() {
                id = Some(i.parse::<usize>().unwrap());
            },
            _ => {
                eprintln!("Unknown option {:?}", arg);
                std::process::exit(1);
            }
        }
    }
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
    if let (Some(id), Some(ref source), Some(ref query_user)) = (id, &*SOURCE, &*QUERY_USER) {
        let db = Source::new(source, query_user, &*QUERY_JOURNAL).unwrap();
        match db.fetch(id).unwrap() {
            Some((r, u)) => match sync.upsert(r, &u) {
                Ok(r) => println!("{:?}: {:?}", r, u),
                Err(e) => eprintln!("Upsert error {:?}: {:?}", e, u),
            },
            None => println!("User {:?} not found", id),
        }
    } else if let (Some(r), Some(u)) = (role, data) {
        match sync.upsert(r, &u) {
            Ok(r) => println!("{:?}: {:?}", r, u),
            Err(e) => eprintln!("Upsert error {:?}: {:?}", e, u),
        }
    }
}
