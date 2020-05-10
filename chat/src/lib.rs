#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

mod models;
mod schema;
use models::{Channel, Id, Membership, Message, User};
use schema::{channels, memberships, messages, users};

use chrono::Utc;
use diesel::{
    insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use failure::{format_err, Error};
use std::env;

pub struct Api {
    conn: PgConnection,
}

impl Api {
    /// Establish DB connection
    pub fn connect() -> Result<Self, Error> {
        let default_url = "postgres://postgres:password@localhost:5432".to_string();
        let database_url = env::var("DATABASE_URL").unwrap_or(default_url);
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    /// Register a new user
    pub fn register_user(&self, email: &str) -> Result<User, Error> {
        insert_into(users::table)
            .values(users::email.eq(email))
            .returning((users::id, users::email))
            .get_result(&self.conn)
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
