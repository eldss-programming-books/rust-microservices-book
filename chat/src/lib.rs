#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use crate::models::{Channel, Id, Membership, Message, User};
use crate::schema::{channels, memberships, messages, users};
use chrono::Utc;
use diesel::{
    insert_into, Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use failure::{format_err, Error};
use std::env;

mod models;
mod schema;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
