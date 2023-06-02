// @generated automatically by Diesel CLI.

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
    session_tokens,
    users,
    video_donation_status,
);
