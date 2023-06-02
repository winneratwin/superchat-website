-- Your SQL goes here

CREATE TABLE video_donation_status (
    -- varchar to store youtube video id
    -- like: JkZ_fq5iXMY
    -- some playlists have a 22 char id
    -- so we'll use 30 to be safe
    id VARCHAR(30) PRIMARY KEY,
    -- we'll store the status as a serialized string
    -- which is a list of booleans
    value Text NOT NULL
);