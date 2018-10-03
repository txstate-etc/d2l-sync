use schemas::{UserBase, Role, ParseError};
use mysql::{Pool, Value};
use mysql::error::Error;
use std::str::FromStr;
use std::env;

lazy_static! {
    static ref QUERY_JOURNAL_MAX_ID: String = {
        match env::var("D2L_QUERY_JOURNAL_MAX_ID") {
            Ok(q) => q,
            Err(_) => panic!("D2L_QUERY_JOURNAL_MAX_ID environment variable not defined"),
        }
    };
}

lazy_static! {
    static ref QUERY_JOURNAL: String = {
        match env::var("D2L_QUERY_JOURNAL") {
            Ok(q) => q,
            Err(_) => panic!("D2L_QUERY_JOURNAL environment variable not defined"),
        }
    };
}

lazy_static! {
    static ref QUERY_USER: String = {
        match env::var("D2L_QUERY_USER") {
            Ok(q) => q,
            Err(_) => panic!("D2L_QUERY_USER environment variable not defined"),
        }
    };
}


pub struct Source {
    pool: Pool,
}

impl Source {
    pub fn new(uri: &str) -> Result<Source, Error> {
        Ok(Source{
            pool: Pool::new(uri)?,
        })
    }

    pub fn journal_max_id(&self) -> Result<Option<usize>, Error> {
        let mut query_journal_max_id = self.pool.prepare(&*QUERY_JOURNAL_MAX_ID)?;
        for row in query_journal_max_id.execute(())? {
            let msn = mysql::from_row::<usize>(row?);
            return Ok(Some(msn));
        }
        Ok(None)
    }

    // returns a vector of (Journal Sequence Number, Option<Internal User ID>)
    pub fn journal(&self, start: usize, limit: usize) -> Result<Option<Vec<(Option<usize>, Option<usize>)>>, Error> {
        let mut query_journal = self.pool.prepare(&*QUERY_JOURNAL)?;
        let mut events = Vec::new();
        for row in query_journal.execute((start, start+limit, start, start+limit))? {
            let (sn, id) = mysql::from_row::<(usize, Option<usize>)>(row?);
            events.push((Some(sn), id));
        }
        if events.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(events))
        }
    }

    pub fn user(&self, user: usize) -> Result<Option<(Role, UserBase)>, Error> {
        let mut query_user = self.pool.prepare(&*QUERY_USER)?;
        for row in query_user.execute((user,))? {
            let (preferred, first, middle, last, user, id, email, role) = mysql::from_row::<(Option<String>, String, String, String, String, String, String, String)>(row?);
            let mut user_base = UserBase::default();
            if let Some(preferred) = preferred {
                user_base.first_name = preferred;
            } else {
                user_base.first_name = first;
                user_base.middle_name = middle;
            }
            user_base.last_name = last;
            user_base.user_name = user;
            user_base.org_defined_id = Some(id);
            user_base.external_email = Some(email);
            return Ok(Some((Role::from_str(&role)?, user_base)));
        }
        Ok(None)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::FromValueError(Value::from(format!("{:?}", err)))
    }
}
