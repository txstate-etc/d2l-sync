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
mod schemas;

use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::str::FromStr;

use schemas::{UserBase, Role};
use source::Source;
use sync::Sync;
use reqwest::Client;

const SEQNUM_LIMIT: usize = 5000;

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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut args = args.iter();
    let mut single_pass_flag = false;
    let mut ids: Option<Vec<(Option<usize>, Option<usize>)>> = None;
    let mut data: Option<UserBase> = None;
    let mut role: Option<Role> = Some(Role::Student);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--ids" => if let Some(is) = args.next() {
                let mut list = Vec::new();
                for i in is.split(",") {
                    list.push((None, Some(i.parse::<usize>().unwrap())));
                }
                if list.len() == 0 {
                    eprintln!("No id values assigned to ids");
                    std::process::exit(1);
                }
                single_pass_flag = true;
                ids = Some(list);
            },
            "-d" | "--data" => if let Some(d) = args.next() {
                data = Some(serde_json::from_str(&d).unwrap());
            },
            "-r" | "--role"  => if let Some(r) = args.next() {
                role = Some(Role::from_str(&r).unwrap());
            },
            _ => {
                eprintln!("Unknown option {:?}", arg);
                std::process::exit(1);
            }
        }
    }
    // WARN: if submitting data then must also provide a role. Since we have a default role then
    // this is not an issue, however, should we remove the default role of Student then this check
    // will be required.
    //if data.is_some() && role.is_none() {
    //    eprintln!("Error: A data update request requires a role must also be specified");
    //    std::process::exit(1);
    //}

    // Setup backend database. NOTE: do NOT always need to query the journal
    // if a list of internal IDs to update is provided via the command line.
    // Also no backend database is required if a specific user and role is
    // provided for a single update.
    let db = if let Some(ref source) = &*SOURCE {
        Some(Source::new(source).unwrap())
    } else {
        None
    };

    // Setup http client syncing module
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

    // Check for single data/role request
    if let (Some(r), Some(u)) = (role, data) {
        match sync.upsert(r, &u) {
            Ok(update_type) => println!("{:?}: {:?}", update_type, u),
            Err(e) => {
                eprintln!("Error: Upsert error {:?}: {:?}", e, u);
                std::process::exit(1);
            },
        }
        std::process::exit(0);
    }

    // All future request types require backend database access to fulfill upsert requests
    if let Some(ref db) = db {
        // Pull sequence number
        //let mut seqnum = db.get_seqnum();
        let mut seqnum = 0;

        // Pull list of ids from commmand line
        let mut events = match ids {
            ids@Some(_) => Ok(ids),
            // Pull list of ids from journal events
            None => db.journal(seqnum, SEQNUM_LIMIT),
        };

        loop {
            match events {
                Ok(Some(is)) => for (sn, uid) in is {
                    if let Some(uid) = uid {
                        match db.user(uid) {
                            Ok(Some((r, ub))) => match sync.upsert(r, &ub) {
                                Ok(update_type) => {
                                    println!("{:?}: {:?}", update_type, uid);
                                    if let Some(sn) = sn {
                                        seqnum = sn;
                                    }
                                }
                                Err(e) => eprintln!("Error: Upsert error {:?}: {:?}", e, ub),
                            },
                            Ok(None) => {
                                println!("User {:?} not found", uid);
                                if let Some(sn) = sn {
                                    seqnum = sn;
                                }
                            },
                            Err(e) => {
                                eprintln!("Error: Database fetch error {:?}: {:?}", uid, e);
                                break;
                            }
                        }
                    } else {
                        if let Some(sn) = sn {
                            seqnum = sn;
                        }
                    }
                },
                Ok(None) => (),
                Err(e) => {
                    eprintln!("Error: Database events error {:?}", e);
                    if !single_pass_flag {
                        sleep(Duration::from_secs(55));
                    }
                },
            }
            if single_pass_flag {
                break;
            }
            //db.set_seqnum(seqnum);
            sleep(Duration::from_secs(5));
            events = db.journal(seqnum, SEQNUM_LIMIT);
        }

    } else {
        eprintln!("Error: D2L_SOURCE Database connection URI is required");
        std::process::exit(1);
    }
}
