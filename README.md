this takes files in a git ignored folder /chat

an instance is hosted on my [website](https://pippasupers.codecoffin.com)

the folder structure is 
```
/chat
    -> channel
        -> video_id
            -> stream_name.donations.json
            -> stream_name.webp
            -> stream_name.date
```
and a .env file

```
DATABASE_URL=postgres://user:password@IpOrDomain:port/pippadonations
```


the donations.json file is made using another one of my programs
in another repo [here](https://github.com/winneratwin/superchat-extractor)
install it using `cargo install --path .`

helper scripts can be found in scripts folder which is run inside
the metadata folder that is created by download-info.sh

if you want to add a livestream to the livestreams part in /streams this is the command i use to watch the live chat for a stream and output donations into a directory
```
STREAMCHAT=./_qpo70DTrak/🔴SURPRISE\ GUEST\ APPEAREANCE\!\ link\ in\ description\!.live_chat.json.part; echo "$STREAMCHAT" | entr superchat-extractor --file $STREAMCHAT 2> ~/rust/superchat-extractor-web/live/"$(basename "$STREAMCHAT" .live_chat.json.part)".donations.json
```
the second part may be any name that ends in .donation.json inside the live folder of the project
