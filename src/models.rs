use diesel::prelude::*;
use std::time::SystemTime;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::session_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SessionToken {
    pub token: String,
    pub username: String,
    pub created_at: SystemTime,
}


#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::video_donation_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VideoDonationStatus {
    pub id: String,
    pub channel: String,
    pub value: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::read_status_change_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReadStatusChangeLog {
    pub id: i32,
    pub timestamp: SystemTime,
    pub username: String,
    pub video_id: String,
    pub donation_id: i32,
    pub previous_status: bool,
    pub new_status: bool,
}