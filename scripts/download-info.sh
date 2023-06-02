#write info json file that has everything
#write descriptions
#write thumbnails
#write steam chat
yt-dlp --download-archive archive-new.txt --force-write-archive --write-info-json --write-description --write-thumbnail --skip-download --write-subs -o "metadata/%(id)s/%(title)s.%(ext)s" "https://www.youtube.com/playlist?list=PLkFt5rGUdJnbYc2uJ9HL3FHVd9BbGxgfL"
