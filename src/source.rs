use sync::{UserBase, Role};
use mysql::Pool;
use mysql::error::Error;

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

    pub fn fetch(&self, user: usize) -> Result<Option<(Role, UserBase)>, Error> {
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
            return Ok(Some((Role::lossy_new(&role), user_base)));
        }
        Ok(None)
    }
    pub fn events(&self, _start: usize, _limit: usize) -> Result<(usize, Option<Vec<String>>), Error> {
        Ok((0, None))
    }
}
