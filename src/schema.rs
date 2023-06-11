// @generated automatically by Diesel CLI.

diesel::table! {
    read_status_change_log (id) {
        id -> Int4,
        timestamp -> Timestamp,
        username -> Text,
        video_id -> Text,
        donation_id -> Int4,
        previous_status -> Bool,
        new_status -> Bool,
    }
}

diesel::table! {
    session_tokens (token) {
        token -> Text,
        username -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (username) {
        username -> Text,
        password -> Text,
    }
}

diesel::table! {
    video_donation_status (id) {
        #[max_length = 30]
        id -> Varchar,
        value -> Text,
    }
}

diesel::joinable!(session_tokens -> users (username));

diesel::allow_tables_to_appear_in_same_query!(
    read_status_change_log,
    session_tokens,
    users,
    video_donation_status,
);
