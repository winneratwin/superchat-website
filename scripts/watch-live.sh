STREAMCHAT=$1;
OUTPUT_FILE=~/rust/superchat-extractor-web/live/$( basename "$STREAMCHAT" .live_chat.json.part).donations.json
echo "Watching $STREAMCHAT";
echo "Outputting to $OUTPUT_FILE";

echo "$STREAMCHAT" | entr superchat-extractor --file "$STREAMCHAT" --dontprint --outputfile "$OUTPUT_FILE"
