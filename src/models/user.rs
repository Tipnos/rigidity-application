use crate::schema::users;
use crate::schema::users::dsl::*;
use crate::diesel::prelude::*;
use crate::chrono::NaiveDateTime;
use diesel::{PgConnection};
use serde::{Deserialize, Serialize};
use crate::models::ORMResult;

// Kept on diesel only for src/models/custom_room.rs's join with `users` and
// the two by-id reads in the custom_room domain (services/custom_room.rs,
// handlers/custom_room/dtos.rs). Everything else about `users` has moved to
// the sqlx-backed crate::database::users module.
#[derive(Serialize, Deserialize, Queryable, AsChangeset)]
#[changeset_options(treat_none_as_null="true")]
pub struct User {
    pub id: i32,
    pub email: String,
    pub nickname: String,
    #[serde(skip_serializing)]
    pub hash: String,
    #[serde(skip_serializing)]
    pub reset_password_hash: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash_expire_at: Option<NaiveDateTime>,
    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,
    pub steam_id: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDateTime,
    #[serde(skip_serializing)]
    pub email_confirmation_required: bool,
}

pub fn get(
    i_d: &i32,
    conn: &PgConnection
) -> ORMResult<User> {
    users.filter(id.eq(i_d))
        .get_result::<User>(conn)
}
