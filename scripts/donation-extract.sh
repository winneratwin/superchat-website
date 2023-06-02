# get directory of where this script was called from
DIR="$( pwd )"

# get all files matching glob */*.live_chat.json
# and run superchat-extractor --file on them
# and pass stderr to */*.donations.json
for chat_log in */*.live_chat.json
do 
	# get directory of chat_log
	videoid=$(dirname -- "$chat_log")
	# get filename of chat_log without extension
	filename=$(basename -- "$chat_log" .live_chat.json)

	# check if .donations.json already exists
	if [ -f "$videoid/$filename.donations.json" ]; then
		#echo "Donations already extracted from $chat_log"
		# head the file to preview it
		#head -n 1 -- "$videoid/$filename.donations.json"
		continue
	fi
	echo "Extracting donations from $chat_log"
	echo "video id: $videoid"
	echo "stream name: $filename"

	superchat-extractor --file "$chat_log" 2> "$videoid/$filename.donations.json"
done
