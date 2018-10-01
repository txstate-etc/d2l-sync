use schemas::{UserBase, Role};
use mysql::{Pool, Value};
use mysql::error::{Error, DriverError};
use std::str::FromStr;
use std::string::ToString;

pub struct Source {
    pool: Pool,
    query_user: String,
    query_journal: Option<String>,
}

impl Source {
    pub fn new(uri: &str, query_user: &str, query_journal: &Option<String>) -> Result<Source, Error> {
        Ok(Source{
            pool: Pool::new(uri)?,
            query_user: query_user.to_string(),
            query_journal: query_journal.clone(),
        })
    }

    // returns a vector of (Journal Sequence Number, Option<Internal User ID>)
    pub fn events(&self, start: usize, limit: usize) -> Result<Option<Vec<(Option<usize>, Option<usize>)>>, Error> {
        if let Some(ref query_journal) = self.query_journal {
            let mut query_journal = self.pool.prepare(query_journal)?;
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
        } else {
            Err(Error::DriverError(DriverError::MissingNamedParameter("No parameters listed".to_string())))
        }
    }

    pub fn query(&self, user: usize) -> Result<Option<(Role, UserBase)>, Error> {
        let mut query_user = self.pool.prepare(&self.query_user)?;
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
            return Ok(Some((Role::from_str(&role).map_err(|e| Error::FromValueError(Value::from(format!("{:?}", e))))?, user_base)));
        }
        Ok(None)
    }
}
