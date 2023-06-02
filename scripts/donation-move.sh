# get directory of where this script was called from
DIR="$( pwd )"

# get all files matching glob */*.donations.json
for chat_log in */*.donations.json; 
do 
	# get directory of chat_log
	videoid=$(dirname -- "$chat_log")
	# get filename of chat_log without extension
	filename=$(basename -- "$chat_log" .donations.json)


	# copy filename.webp, filename.donations.json, filename.info.json to a directory named out
	# check if directory exists, if not, create it
	if [ ! -d "$DIR/out/$videoid" ]; then
		echo "video id: $videoid"
		echo "stream name: $filename"
		echo "Creating directory $DIR/out/$videoid"
		mkdir -p "$DIR/out/$videoid"
		cp -- "$videoid/$filename.webp" "$DIR/out/$videoid/$filename.webp"
		cp -- "$videoid/$filename.donations.json" "$DIR/out/$videoid/$filename.donations.json"
		jq .epoch -- "$videoid/$filename.info.json" > "$DIR/out/$videoid/$filename.date"
	# else print error message
	else
		#echo "Directory $DIR/out/$videoid already exists"
		echo "Skipping $videoid"
	fi

done
