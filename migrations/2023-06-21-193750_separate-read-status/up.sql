-- Your SQL goes here


-- add column to video_donation_status
ALTER TABLE "video_donation_status" ADD COLUMN "channel" TEXT NOT NULL DEFAULT 'pippa';
-- make it a primary key
ALTER TABLE "video_donation_status" DROP CONSTRAINT "video_donation_status_pkey";
ALTER TABLE "video_donation_status" ADD PRIMARY KEY ("channel", "id");