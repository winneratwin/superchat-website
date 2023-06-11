-- Your SQL goes here



CREATE TABLE "read_status_change_log"(
	"id" INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	"timestamp" TIMESTAMP NOT NULL,
	"username" TEXT NOT NULL,

	"video_id" TEXT NOT NULL,
	"donation_id" INTEGER NOT NULL,
	"previous_status" BOOLEAN NOT NULL,
	"new_status" BOOLEAN NOT NULL
);

