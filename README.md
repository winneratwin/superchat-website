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
