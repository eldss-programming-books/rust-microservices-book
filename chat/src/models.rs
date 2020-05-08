use crate::schema::{channels, memberships, messages, users};
use chrono::NaiveDateTime;

pub type Id = i32;

/// A User of the chat service
#[derive(Debug, Identifiable, Queryable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: Id,
    pub email: String,
}

/// A channel is where chatting is done
#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(User)]
#[table_name = "channels"]
pub struct Channel {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// A User has a membership to a channel
#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "memberships"]
pub struct Membership {
    pub id: Id,
    pub channel_id: Id,
    pub user_id: Id,
}

/// A message is sent by a user to a channel
#[derive(Debug, Identifiable, Queryable, Associations, Serialize, Deserialize)]
#[belongs_to(Channel)]
#[belongs_to(User)]
#[table_name = "messages"]
pub struct Message {
    pub id: Id,
    pub timestamp: NaiveDateTime,
    pub channel_id: Id,
    pub user_id: Id,
    pub text: String,
}
