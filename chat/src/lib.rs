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

    /// Create a new channel
    pub fn create_channel(
        &self,
        user_id: Id,
        title: &str,
        is_public: bool,
    ) -> Result<Channel, Error> {
        self.conn.transaction::<_, _, _>(|| {
            // Create the channel
            let channel: Channel = insert_into(channels::table)
                .values((
                    channels::user_id.eq(user_id),
                    channels::title.eq(title),
                    channels::is_public.eq(is_public),
                ))
                .returning((
                    channels::id,
                    channels::user_id,
                    channels::title,
                    channels::is_public,
                    channels::created_at,
                    channels::updated_at,
                ))
                .get_result(&self.conn)
                .map_err(Error::from)?;

            // Add the user who created it to the channel
            self.add_member(channel.id, user_id)?;

            Ok(channel)
        })
    }

    /// Make a given channel public
    pub fn publish_channel(&self, channel_id: Id) -> Result<(), Error> {
        let channel = channels::table
            .filter(channels::id.eq(channel_id))
            .select((
                channels::id,
                channels::user_id,
                channels::title,
                channels::is_public,
                channels::created_at,
                channels::updated_at,
            ))
            .first::<Channel>(&self.conn)
            .optional()
            .map_err(Error::from)?;

        if let Some(channel) = channel {
            diesel::update(&channel)
                .set(channels::is_public.eq(true))
                .execute(&self.conn)?;
            Ok(())
        } else {
            Err(format_err!("channel not found"))
        }
    }

    /// Adds a user as a member to a channel
    pub fn add_member(&self, channel_id: Id, user_id: Id) -> Result<Membership, Error> {
        insert_into(memberships::table)
            .values((
                memberships::channel_id.eq(channel_id),
                memberships::user_id.eq(user_id),
            ))
            .returning((
                memberships::id,
                memberships::channel_id,
                memberships::user_id,
            ))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    /// Add a message from the given user to the given channel with the given content/text
    pub fn add_message(&self, channel_id: Id, user_id: Id, text: &str) -> Result<Message, Error> {
        let timestamp = Utc::now().naive_utc();
        insert_into(messages::table)
            .values((
                messages::timestamp.eq(timestamp),
                messages::channel_id.eq(channel_id),
                messages::user_id.eq(user_id),
                messages::text.eq(text),
            ))
            .returning((
                messages::id,
                messages::timestamp,
                messages::channel_id,
                messages::user_id,
                messages::text,
            ))
            .get_result(&self.conn)
            .map_err(Error::from)
    }

    /// Remove a message from a channel
    pub fn delete_message(&self, message_id: Id) -> Result<(), Error> {
        diesel::delete(messages::table)
            .filter(messages::id.eq(message_id))
            .execute(&self.conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Api;

    #[test]
    fn create_users() {
        let api = Api::connect().unwrap();

        // Create users
        let user1 = api.register_user("user1@example.com").unwrap();
        let user2 = api.register_user("user2@example.com").unwrap();

        // Create channel and make public
        let channel = api.create_channel(user1.id, "My Channel", false).unwrap();
        api.publish_channel(channel.id).unwrap();

        // Add user2 to channel
        api.add_member(channel.id, user2.id).unwrap();

        // Send some messages
        let message = api.add_message(channel.id, user1.id, "Welcome").unwrap();
        api.add_message(channel.id, user2.id, "Hi!").unwrap();

        // remove the first message
        api.delete_message(message.id).unwrap();
    }
}
