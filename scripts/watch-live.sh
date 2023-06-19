STREAMCHAT=$1;
OUTPUT_FILE=~/rust/superchat-extractor-web/live/$( basename "$STREAMCHAT" .live_chat.json.part).donations.json

inotifywait -m -e modify "$STREAMCHAT" |
while read file_event
do 
  superchat-extractor --file "$STREAMCHAT" --dontprint --outputfile "$OUTPUT_FILE"
  echo "Updated $OUTPUT_FILE"
done
