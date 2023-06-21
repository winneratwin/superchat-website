-- This file should undo anything in `up.sql`



ALTER TABLE "video_donation_status" DROP COLUMN "channel";
ALTER TABLE "video_donation_status" DROP CONSTRAINT "video_donation_status_pkey";
ALTER TABLE "video_donation_status" ADD PRIMARY KEY ("id");
