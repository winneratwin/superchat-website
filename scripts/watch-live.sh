STREAMCHAT=$1;
OUTPUT_FILE=~/rust/superchat-extractor-web/live/$( basename "$STREAMCHAT" .live_chat.json.part).donations.json

inotifywait -m -e modify,move_self "$STREAMCHAT" --format="%e %w%f"|
while read -r event_name file_name;
do 
  #echo "Event: $event_name on $file_name"
  case "$event_name" in
    MOVE_SELF)
      #echo "Moved from $file_name"
      echo "stream ended"
      superchat-extractor --file "$STREAMCHAT" --end
      ;;
    MODIFY)
      #echo "Modified $file_name"
      superchat-extractor --file "$STREAMCHAT" --dontprint --live
      ;;
    *)
      echo "Unknown event $event_name"
      ;;
  esac
done
