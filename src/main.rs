use actix_web::{get, post, App, web, HttpResponse, HttpServer, Responder};
use askama_actix::{Template,TemplateToResponse};
use glob::glob;
use actix_files as fs;
use urlencoding::encode;
use std::io::{Seek,Read,BufRead};
use minify_html::{Cfg, minify};
use interprocess::local_socket::{LocalSocketListener};

use diesel::prelude::*;
pub mod schema;
pub mod models;

struct Stream {
	title: String,
	thumbnail: String,
	link: String,
	date_released: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate{
}

#[get("/")]
async fn index() -> impl Responder {
	let template = IndexTemplate{};
	template.to_response()
}


#[derive(Template)]
#[template(path = "streams.html")]
struct StreamsTemplate<'a> {
	streams: &'a Vec<Stream>,
}


#[derive(Template)]
#[template(path = "streamers.html")]
struct ChannelsTemplate {
	channels: Vec<String>,
	streams: Vec<(String,String)>,
}

#[get("/streams")]
async fn streamers(server: web::Data<Addr<ChatServer>>) -> impl Responder {
	let mut out = Vec::new();
	// loop over chats directory
	let paths: Vec<_> = std::fs::read_dir("./chats").expect("Unable to read chat directory")
		.map(Result::unwrap)
		.collect();
	for file in paths {
		// check if file is a directory
		if !file.file_type().expect("failed to read filetype").is_dir() {
			continue;
		}

		// convert to std::Path
		let path = file.path();

		// get folder_name
		let folder_name = path.file_name().expect("failed to read filename").to_str().expect("failed to convert filename to str").to_string();

		out.push(folder_name);
	}

	// get streams
	let stream_names = match server.send(GetStreams{}).await{
		Ok(res) => res,
		Err(_) => return HttpResponse::InternalServerError().finish(),
	};
	
	let stream_names = stream_names.iter().map(|s| s.strip_prefix("live-").expect("failed to strip prefix")).map(|s| (s.to_string() ,encode(s).into_owned())).collect::<Vec<(String,String)>>();


	let template = ChannelsTemplate{channels: out, streams: stream_names};
	template.to_response()
}



#[get("/streams/{channel_name}")]
async fn streams(channel_name: web::Path<String>) -> impl Responder {
	let mut out: Vec<Stream> = Vec::new();
	// loop over chats directory
	let paths: Vec<_> = std::fs::read_dir(format!("./chats/{channel_name}")).expect("Unable to read chat directory")
		.map(|r| r.expect("failed to read file"))
		.collect();
	for file in paths {
		// check if file is a directory
		if !file.file_type().expect("failed to read filetype").is_dir() {
			continue;
		}

		// convert to std::Path
		let path = file.path();

		// get folder_name
		let folder_name = path.file_name().expect("failed to read filename").to_str().expect("failed to convert filename to str").to_string();

		// get thumbnail
		let thumbnail = glob(&format!("./chats/{channel_name}/{folder_name}/*.webp")).expect("Failed to read thumbnail").next().expect("Failed to read thumbnail").expect("Failed to read thumbnail");
		let thumbnail_filename = thumbnail.as_path().file_name().expect("Failed to read thumbnail").to_str().expect("Failed to convert thumbnail to str").to_string();
		let thumbnail_filename = encode(&thumbnail_filename).into_owned();
		// remove file extension
		let stream_name = thumbnail.file_stem().expect("Failed to read file name").to_str().expect("Failed to convert file name to str").to_string();
		
		// get 
		let date_file_path = format!("./chats/{channel_name}/{folder_name}/{stream_name}.date");
		let date_file = std::path::Path::new(&date_file_path);

		// parse date_file as i64
		// contents are just a number
		let date_released_string = std::fs::read_to_string(date_file).expect("Failed to read info file");

		/* 
		let desc_file_path = format!{"./chats/{}/{}.description",folder_name,stream_name};
		let desc_file = std::path::Path::new(&desc_file_path);
		let desc_file_name = desc_file.file_name().expect("failed to get filename").to_str().expect().to_string();
		*/
		let res = Stream{
			title: stream_name,
			thumbnail: format!("/files/{channel_name}/{folder_name}/{thumbnail_filename}"),
			link: format!("/chat/{channel_name}/{folder_name}"),
			date_released: date_released_string,
		};
		out.push(res);
	}

	// sort by date released desc
	out.sort_by(|a,b| b.date_released.cmp(&a.date_released));
	
	let template = StreamsTemplate { streams: &out };
	template.to_response()
}


//function to convert number to rgba
fn toargb(number:i64) -> String {
	let a = number >> 24 & 255;
	let r = number >> 16 & 255;
	let g = number >> 8 & 255;
	let b = number & 255;
	if a == 0 {
		format!("rgb({r},{g},{b})")
	} else {
		format!("rgba({r},{g},{b},{a})")
	}
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Donation {
	username: String,
	channel_id: String,
	amount: String,
	message: Option<String>,
	time: String,
	header_color: i64,
	body_color: i64,
	thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
struct Membership {
	username: String,
	channel_id: String,
	months: String,
	message: Option<String>,
	time: String,
	header_color: i64,
	body_color: i64,
	thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Redemption {
	thumbnail_url: String,
	username: String,
	channel_id: String,
	sender: String,
	time: String,
	header_color: i64,
	body_color: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Sticker {
	username: String,
	channel_id: String,
	sticker_cost: String,
	sticker_description: String,
	sticker_image_url: String,
	thumbnail_url: String,
	time: String,
	header_color: i64,
	body_color: i64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Gift {
	username: String,
	channel_id: String,
	number: String,
	time: String,
	header_color: i64,
	body_color: i64,
	thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum DonationTypes {
	Donation(Donation),
	Membership(Membership),
	GiftMembership(Redemption),
	GiftingMembership(Gift),
	Sticker(Sticker),

	Unknown(serde_json::Value),
}

macro_rules! convert_int_to_color {
	($x:expr) => {
		toargb($x)
	};
}


#[derive(Template)]
#[template(path = "donations.html")]
struct DonationsTemplate {
	donations: Vec<String>,
	colors: Vec<(i64,i64)>,
	video_id: String,
	channel_name: String,
}

#[derive(Template)]
#[template(path = "donation.html")]
struct DonationTemplate {
	donation: DonationTypes,
	donation_id: i32,
}

#[get("/chat/{channel_name}/{chat_name}")]
async fn chat(path: web::Path<(String,String)>,server: web::Data<Addr<ChatServer>>) -> impl Responder {
	// print chat name
	
	let (channel_name,chat_name) = path.into_inner();
	//println!("chat name: {}",chat_name);

	// if channel_name is live then chat_name is the video_name
	if channel_name == "live" {
		// get running streams from server
		let stream_names = match server.send(GetStreams{}).await {
			Ok(res) => res,
			Err(_) => return HttpResponse::InternalServerError().body("Failed to get streams from server"),
		};
		// check if stream exists
		// add live- to the front of the stream name
		if !stream_names.contains(&format!("live-{}",chat_name)) {
			return HttpResponse::NotFound().body("<main>
	<div class=\"m-2\">
		<h1 class=\"text-2xl\">Error</h1>
		<p>no stream with that name</p>
	</div>
</main>");
		}

		

		let donations = Vec::new();
		let donations_colors = Vec::new();
		let template = DonationsTemplate { donations, colors: donations_colors, video_id: format!("live-{}",encode(&chat_name)), channel_name };
		return template.to_response()
	}

	// check if chat exists
	let chat_path = format!("./chats/{channel_name}/{chat_name}");
	if !std::path::Path::new(&chat_path).exists() {
		return HttpResponse::NotFound().body("<main>
	<div class=\"m-2\">
		<h1 class=\"text-2xl\">Error</h1>
		<p>Failed to read donations file</p>
		<p>might not exist on the server</p>
	</div>
</main>");
	}
	// get donations file
	// ends in .donations.json
	let donations_file = glob(&format!("./chats/{channel_name}/{chat_name}/*.donations.json")).expect("Failed to read donations file").next().expect("Failed to read donations file").expect("Failed to read donations file");
	// read donations file
	let donations_file = std::fs::read_to_string(donations_file).expect("Failed to read donations file");

	// get donations from donations file
	let mut donations = Vec::new();
	let mut donations_colors = Vec::new();
	// for each donation
	for (id,line) in donations_file.lines().enumerate() {
		// parse donation
		let donation: DonationTypes = serde_json::from_str(line).expect("Failed to parse donation");
		match &donation {
			DonationTypes::Donation(donation) => {
				donations_colors.push((donation.header_color,donation.body_color));
			},
			DonationTypes::Sticker(sticker) => {
				donations_colors.push((sticker.header_color,sticker.body_color));
			},
			_ =>{}
		}
		let donation = DonationTemplate{ donation, donation_id: i32::try_from(id).expect("failed to convert id") }.render().expect("Failed to render donation");
		let donation = String::from_utf8(minify(donation.as_bytes(), &Cfg::default())).expect("Failed to convert to string");
		donations.push(donation);
	}
	//println!("donations: {:?}",donations);
	// deduplicate colors
	donations_colors.sort_unstable();
	donations_colors.dedup();
	//println!("donations_colors: {:?}",donations_colors);
	let template = DonationsTemplate { donations, colors: donations_colors, video_id: chat_name, channel_name };
	template.to_response()
	

	// echo chat name
	//HttpResponse::Ok().body(format!("chat name: {}",chat_name))
}

use actix::prelude::*;
use actix_web::{Error, HttpRequest};
use actix_web_actors::ws;
use serde_json::{json, Value};

#[get("/ws/{channel_name}/{chat_name}")]
async fn ws_index(r: HttpRequest, stream: web::Payload, path: web::Path<(String,String)>, server: web::Data<Addr<ChatServer>>) -> Result<HttpResponse, Error> {
	let (streamer, chat_name) = path.into_inner();
	// if chat_name starts with live- then it is a live chat
	let is_live = chat_name.starts_with("live-");
	// if it is live then check if file exists in live folder
	if is_live {
		let chat_name = chat_name.strip_prefix("live-").expect("Failed to strip prefix");
	}

	let client_id = uuid::Uuid::new_v4().to_string();

	let actor = MyWebSocket { addr: server.get_ref().clone(), room: chat_name, is_user: false, hb: Instant::now(), username:String::new(), id: client_id, channel_name: streamer };

    ws::start(actor, &r, stream)
}

use std::time::SystemTime;
#[post("/login")]
async fn login(body:web::Json<AuthenticationContents>, pool_wrap: web::Data<Pool<ConnectionManager<PgConnection>>> ) -> impl Responder {
	use crate::schema::users::dsl::{users, username};
	// get pool
	let mut pool = pool_wrap.get().expect("Failed to get connection from pool");
	// check if user exists
	let user = users.filter(username.eq(&body.username)).first::<models::User>(&mut pool);
	match user {
		Ok(user) => {
			// check if password is correct
			if user.password == body.password {
				// generate token
				let token = uuid::Uuid::new_v4().to_string();
				// get current time
				let now = SystemTime::now();

				//insert token into database
				diesel::insert_into(schema::session_tokens::table)
					.values((
						schema::session_tokens::username.eq(&body.username),
						schema::session_tokens::token.eq(&token),
						schema::session_tokens::created_at.eq(&now),
					))
					.execute(&mut pool)
					.expect("Error saving new token");

				// return success
				HttpResponse::Ok().json(json!({"success":true,"token":token}))
			} else {
				// return failure
				HttpResponse::Ok().json(json!({"success":false,"reason":"incorrect password"}))
			}
		},
		Err(_) => {
			// return failure
			HttpResponse::Ok().json(json!({"success":false,"reason":"user does not exist"}))
		}
	}
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
}

#[get("/login")]
async fn login_page() -> impl Responder {
	// return login page
	let template = LoginTemplate {};
	template.to_response()
}

// the websocket actor
struct MyWebSocket {
	// parent
	addr: Addr<ChatServer>,
	room: String,
	channel_name: String,
	id: String,
	is_user: bool,
	username: String,
	// heartbeat
	hb: Instant,
}

#[derive(Message)]
#[rtype(result = "()")]
struct GetDonations {
	chat_name: String,
	channel_name: String,
	addr: Addr<MyWebSocket>,
}

impl Handler<GetDonations> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: GetDonations, _ctx: &mut Self::Context) -> Self::Result {

		// check if it is a live stream by checking if it begins
		// with live-
		if msg.chat_name.starts_with("live-") {
			// get donations from self.live_donations and the corresponding
			// read status from self.read_status
			if let Some(res) = self.live_donations.get(&msg.chat_name) {
				if let Some(status) = self.read_status.get(&msg.chat_name) {
					//println!("sending donations from cache");
					for (id,donation) in res.iter().enumerate() {
						let donation = DonationTemplate { donation: donation.clone(), donation_id: i32::try_from(id).expect("failed to convert id") }.render().expect("Failed to render donation");
						let donation = String::from_utf8(minify(donation.as_bytes(), &Cfg::default())).expect("Failed to convert to string");

						msg.addr.do_send(NewMessageForClient{ donation, read_status: status[id] });
					}
					return;
				}
			}
			return;
		}

		// check if it has already been queried
		// we store the results from the database in the actor
		// so we don't have to query the database every time
		// we want to get the donations. we only insert new
		// into the database on update
		
		// check self.read_status to see if there is an entry
		// if there is, send it to the client
		// if there isn't, query the database
		if let Some(res) = self.read_status.get(&msg.chat_name) {
			//println!("sending donations from cache");
			for (id,status) in res.iter().enumerate() {
				msg.addr.do_send(ReadUpdate{ donation_id: i32::try_from(id).expect("failed to convert id"), is_read: *status, video_id: msg.chat_name.clone(), username: String::new(), channel_name: msg.channel_name.clone() });
			}
			return;
		}


		// get donations
		// do a database query 
		let mut pool = self.pool.get().expect("Failed to get database connection");
		let videos = schema::video_donation_status::table
			.filter(schema::video_donation_status::id.eq(&msg.chat_name))
			.filter(schema::video_donation_status::channel.eq(&msg.channel_name))
			.load::<models::VideoDonationStatus>(&mut pool)
			.expect("Error loading donations");

		// check if there are any donations
		let exists =  !videos.is_empty();

		if exists {
			for donations in videos {
				let list:Value = serde_json::from_str(donations.value.as_str()).expect("database corruption");
				let list = list.as_array().expect("failed to convert to array").iter().map(|x| x.as_bool().expect("failed to parse boolian")).collect::<Vec<bool>>();
				
				// cache the results
				self.read_status.insert(msg.chat_name.clone(),list.clone());
				
				for (i,x) in list.iter().enumerate() {
					msg.addr.do_send(ReadUpdate{ donation_id: i32::try_from(i).expect("failed to convert i"), is_read: *x, video_id: msg.chat_name.clone(), username: String::new(), channel_name: msg.channel_name.clone() });
				}
			}
		} else {
			//println!("donations not found in database");

			// get donations file
			let donations_file = glob(&format!("./chats/{}/{}/*.donations.json",msg.channel_name,msg.chat_name)).expect("Failed to read donations file").next().expect("Failed to read donations file").expect("Failed to read donations file");
			// read donations file
			let donations_file = std::fs::read_to_string(donations_file).expect("Failed to read donations file").lines().count();
			// create empty list with default value of false
			// with length of numbers of donations
			let list = vec![false;donations_file];

			// cache the results
			self.read_status.insert(msg.chat_name.clone(),list.clone());

			// write to database
			diesel::insert_into(schema::video_donation_status::table)
				.values((
					schema::video_donation_status::id.eq(msg.chat_name.clone()),
					schema::video_donation_status::value.eq(serde_json::to_string(&list).expect("Failed to serialize donations")),
					schema::video_donation_status::channel.eq(msg.channel_name.clone())
				))
				.execute(&mut pool)
				.expect("Error saving new donations");

			// loop through donations
			for (i,v) in list.iter().enumerate() {
				// send update to client
				msg.addr.do_send(ReadUpdate{ donation_id: i32::try_from(i).expect("failed to convert i"), is_read: *v, video_id: msg.chat_name.clone(), username: String::new(), channel_name: msg.channel_name.clone() });
			}
		}
	}
}

use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

	// called when websocket is started
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
		// add self to room
		self.addr.do_send(AddToRoom { chat_name: self.room.clone(), addr: ctx.address(), id: self.id.clone(), channel_name: self.channel_name.clone() });
        println!("Added {} to room {} in channel {}",self.id,self.room,self.channel_name);
		
		// check if database has information about this video
		// by calling the server
		self.addr.do_send(GetDonations { chat_name: self.room.clone(), addr: ctx.address(), channel_name: self.channel_name.clone() });
    }

	// called when websocket is stopped
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        //println!("WebSocket stopped");
		// remove self from room
		self.addr.do_send(RemoveFromRoom { chat_name: self.room.clone(), id: self.id.clone(), channel_name: self.channel_name.clone() });
    }
}

impl MyWebSocket {
    /// helper method that sends ping to client every 5 seconds (`HEARTBEAT_INTERVAL`).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}


// Define a signal that the server can use to add a WebSocket actor to a room
#[derive(Message)]
#[rtype(result = "()")]
struct AddToRoom {
    chat_name: String,
	channel_name: String,
    addr: Addr<MyWebSocket>,
	id: String,
}
// Implement a signal handler for the server actor
impl Handler<AddToRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: AddToRoom, _ctx: &mut Self::Context) -> Self::Result {
        // Get the list of WebSocket actors for the specified room
        let room = self.rooms.entry((msg.chat_name.clone(),msg.channel_name.clone())).or_default();

        // Add the WebSocket actor to the room
        room.insert(msg.id,WebsocketClient { addr: msg.addr });
    }
}

// Define a signal that the server can use to remove a WebSocket actor from a room
#[derive(Message)]
#[rtype(result = "()")]
struct RemoveFromRoom {
    chat_name: String,
	channel_name: String,
    id: String,
}
// Implement a message handler for the server actor
impl Handler<RemoveFromRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RemoveFromRoom, _ctx: &mut Self::Context) -> Self::Result {
        // Get the list of WebSocket actors for the specified room
        if let Some(room) = self.rooms.get_mut(&(msg.chat_name.clone(), msg.channel_name.clone())) {
			// remove the WebSocket actor from the hash map
			if room.remove(&msg.id).is_some() {
				println!("Removed {} from room {} in channel {}",msg.id,msg.chat_name,msg.channel_name);
			}
        }
    }
}


struct WebsocketClient {
	addr: Addr<MyWebSocket>,
}

use std::collections::HashMap;
// Define the server actor
struct ChatServer {
    rooms: HashMap<(String, String), HashMap<String, WebsocketClient>>,
	pool: Pool<ConnectionManager<PgConnection>>,
	read_status: HashMap<String, Vec<bool>>,
	live_donations: HashMap<String, Vec<DonationTypes>>
}

impl Actor for ChatServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Server started");
    }
}

// Define a signal that the server can use to broadcast a message to all WebSocket actors in a room
#[derive(Message,Debug)]
#[rtype(result = "()")]
struct ReadUpdate {
	donation_id:i32,
	is_read:bool,
	video_id:String,
	channel_name:String,
	username:String,
}

#[derive(Message,Debug)]
#[rtype(result = "Vec<String>")]
struct GetStreams {
}

impl Handler<GetStreams> for ChatServer {
	type Result = Vec<String>;

	fn handle(&mut self, _msg: GetStreams, _ctx: &mut Self::Context) -> Self::Result {
		// return keys of self.live_donations
		self.live_donations.keys().cloned().collect()
	}
}


// implement a message handler for the server actor
impl Handler<ReadUpdate> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: ReadUpdate, _ctx: &mut Self::Context) -> Self::Result {
		// get current read status
		if let Some(status) = self.read_status.get(&msg.video_id) {
			let mut pool = self.pool.get().expect("couldn't get db connection from pool");

			// change the status
			let mut status = status.clone();
			let previous_read_status = status[usize::try_from(msg.donation_id).expect("failed to convert donation_id")];
			status[usize::try_from(msg.donation_id).expect("failed to convert donation_id")] = msg.is_read;
			self.read_status.insert(msg.video_id.clone(), status.clone());
			

			// update other clients
			if let Some(room) = self.rooms.get_mut(&(msg.video_id.clone(),msg.channel_name.clone())) {
				for client in room.values() {
					client.addr.do_send(ReadUpdate { donation_id: msg.donation_id, is_read: msg.is_read, video_id: msg.video_id.clone(), username: String::new(), channel_name: msg.channel_name.clone() });
				}
			} else {
				println!("room not found someow. this should not happen");
			}
			
			// if livestream don't update database
			// livestream if starts with live-
			if msg.video_id.starts_with("live-") {
				return;
			}

			std::thread::spawn(move || {
			{ // own namespace to prevent overlap of id variable names
				use crate::schema::read_status_change_log::dsl::{channel, donation_id, new_status, previous_status, read_status_change_log, timestamp, username, video_id};
	
				// log change to read_status_change_log
				// need: timestamp, username, video_id, donation_id, previous_status, new_status
				// username is msg.username
				// video_id is msg.video_id
				// donation_id is msg.donation_id
				// previous_status is previous_read_status
				// new_status is msg.is_read
				// timestamp is SystemTime::now()
	
				
				let _res = diesel::insert_into(read_status_change_log)
					.values((
						timestamp.eq(SystemTime::now()),
						username.eq(&msg.username),
						video_id.eq(&msg.video_id),
						donation_id.eq(&msg.donation_id),
						previous_status.eq(&previous_read_status),
						new_status.eq(&msg.is_read),
						channel.eq(&msg.channel_name),
					))
					.get_result::<models::ReadStatusChangeLog>(&mut pool)
					.expect("Error inserting into read_status_change_log");	
			}
			{
				use crate::schema::video_donation_status::dsl::{id, value};

				// update the database
				let serialized = serde_json::to_string(&status).expect("Error serializing read_status");

				let _res = diesel::update(schema::video_donation_status::table)
					.filter(id.eq(&msg.video_id))
					.set( value.eq(serialized))
					.get_result::<models::VideoDonationStatus>(&mut pool)
					.expect("Error updating video_donation_status");
			}});
		} else {
			println!("client sent update for video not cached");
		};
	}
}

// same for client
impl Handler<ReadUpdate> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, msg: ReadUpdate, ctx: &mut Self::Context) -> Self::Result {
		//println!("ReadUpdate: {:?}",msg);
		// deserialize the message to ReadUpdateContents
		let contents = json!({"donation_id": msg.donation_id, "is_read": msg.is_read, "status":"success", "message":"read status updated"});
		// serialize the message to json
		let json = serde_json::to_string(&contents).expect("Error serializing ReadUpdateContents");
		// send the message to the client
		ctx.text(json);
	}
}

#[derive(Serialize,Deserialize,Debug)]
struct AuthenticationContents {
	username:String,
	password:String
}

#[derive(Message)]
#[rtype(result = "()")]
struct Authentication {
	username:String,
	password:String,
    addr: Addr<MyWebSocket>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct AuthenticationToClient {
	exists:bool,
	username:Option<String>
}


impl Handler<Authentication> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: Authentication, _ctx: &mut Self::Context) -> Self::Result {
		use schema::session_tokens::dsl::{session_tokens, username, token};

		// check if user exists in database
		let mut conn: PooledConnection<ConnectionManager<PgConnection>> = self.pool.get().expect("Error connecting to database");

		let results = session_tokens
			.filter(username.eq(&msg.username))
			.filter(token.eq(&msg.password))
			.load::<models::SessionToken>(&mut conn)
			.expect("Error loading session tokens");
		//println!("results: {:?}",results);

		if let Some(result) = results.first() {
			// send the message to the client
			msg.addr.do_send(AuthenticationToClient { exists: true, username: Some(result.username.clone()) });
		} else{
			// send the message to the client
			msg.addr.do_send(AuthenticationToClient { exists: false, username: None });
		}

	}
}

impl Handler<AuthenticationToClient> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, msg: AuthenticationToClient, ctx: &mut Self::Context) -> Self::Result {
		self.is_user = msg.exists;
		if msg.exists {
			// set username
			self.username = msg.username.expect("username should be passed when authenticated");

			// send success message
			ctx.text(json!({"status":"success","message":"authenticated"}).to_string());
		} else {
			ctx.text(json!({"status":"error","message":"not authenticated"}).to_string());
		}
	}
}


#[derive(Serialize,Deserialize,Debug)]
struct ReadUpdateContents {
	donation_id:i32,
	is_read:bool
}

// incoming messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(text)) => {
				// check if it is a read update
				if let Ok(contents) = serde_json::from_str::<ReadUpdateContents>(&text) {
					if !self.is_user {
						ctx.text(json!({"status":"error","message":"not authenticated"}).to_string());
						return;
					}
					// send the message to the server
					self.addr.do_send(ReadUpdate { donation_id: contents.donation_id, is_read: contents.is_read, video_id: self.room.clone(), username: self.username.clone(), channel_name: self.channel_name.clone() });
				} else if let Ok(contents) = serde_json::from_str::<AuthenticationContents>(&text) {
					// send the message to the server
					self.addr.do_send(Authentication { username: contents.username, password: contents.password, addr: ctx.address() });
				} else {
					
					// return error to client
					ctx.text(json!({"status":"error","message":"invalid json"}).to_string());
				}
			},				
            Ok(ws::Message::Binary(_)) => println!("Unexpected binary"),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
			Err(e) => {
				println!("Error: {e:?}");
				ctx.stop();
			}
        }
    }
}

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

#[derive(Message)]
#[rtype(result = "()")]
struct LiveUpdate {
	livestream_name: String,
	donation: DonationTypes,
}

#[derive(Message)]
#[rtype(result = "()")]
struct NewMessageForClient {
	donation: String,
	read_status: bool,
}

impl Handler<NewMessageForClient> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, msg: NewMessageForClient, ctx: &mut Self::Context) -> Self::Result {
		// send the message to the client
		ctx.text(json!({"status":"success","message":"new donation","donation":msg.donation,"read":msg.read_status}).to_string());
	}
}

impl Handler<LiveUpdate> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: LiveUpdate, _ctx: &mut Self::Context) -> Self::Result {
		// store the donation in the live_donations hashmap
		self.live_donations.entry(msg.livestream_name.clone()).or_default().push(msg.donation.clone());
		// append false value to read_status
		self.read_status.entry(msg.livestream_name.clone()).or_default().push(false);

		// get length of the vector
		let len = self.live_donations.get(&msg.livestream_name).expect("failed to get livestream donations").len();

		// send the message to all clients in the room
		for client in self.rooms.entry((msg.livestream_name,"live".to_string())).or_default().values() {
			let donation = DonationTemplate{ donation: msg.donation.clone(), donation_id: i32::try_from(len-1).expect("failed to convert donation_id")}.render().expect("Failed to render donation template");
			
			let donation = String::from_utf8(minify(donation.as_bytes(), &Cfg::default())).expect("Failed to convert to string");
			client.addr.do_send(NewMessageForClient { donation, read_status:false });
		}
	}
}

#[derive(Message)]
#[rtype(result = "()")]
struct StreamEnd {
	livestream_name: String,
}

impl Handler<StreamEnd> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: StreamEnd, _ctx: &mut Self::Context) -> Self::Result {
		// disconnect all clients in the room
		for client in self.rooms.entry((msg.livestream_name.clone(),"live".to_string())).or_default().values() {
			client.addr.do_send(StreamEnd { livestream_name: msg.livestream_name.clone() });
		}
		// print the read donations
		self.read_status.get(&msg.livestream_name).map_or_else( || {
				println!("Read donations: 0");
			}, |donations| {
				println!("Read donations: {donations:?}");
			});
	}
}

impl Handler<StreamEnd> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, _msg: StreamEnd, ctx: &mut Self::Context) -> Self::Result {	
		// send the message to the client that the stream has ended
		ctx.text(json!({"status":"success","message":"stream ended"}).to_string());
		// disconnect the client
		ctx.stop();
	}
}




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use std::env;
	enum LiveDonationTypes {
		Donation(String, DonationTypes),
		StreamEnd(String),
	}

	/*
	let vec = vec![true,false,true];
	let json_res = json!(vec);
	return Ok(()); */
	// create a pool of database connections
	dotenv::dotenv().ok();
    let postgres_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let manager = ConnectionManager::<PgConnection>::new(postgres_url);
	let pool = Pool::builder().max_size(15).build(manager).expect("Failed to create database connection pool");
	// note to self: actors will freeze when this is used uncomment reader_actor and this to see it
	//let live_messages: Arc<Mutex<HashMap<String, Vec<DonationTypes>>>> = Arc::new(Mutex::new(HashMap::new())); 


    // Create the chat server instance
    let server = ChatServer { rooms: HashMap::new(), pool: pool.clone(), read_status: HashMap::new(), live_donations: HashMap::new() }.start();
	let (main_sender, receiver) = std::sync::mpsc::channel();

	// loop over all files in the live directory
	let live_dir:Vec<_> = std::fs::read_dir("./live").expect("failed to read live directory")
		.map(Result::unwrap)
		.collect();
	for file in live_dir {
		// extract file name
		let file_name = file.file_name().into_string().expect("failed to convert file name to string");
		// check if it ends with .donations.json
		// if not skip it
		if !file_name.ends_with(".donations.json") {
			continue;
		}

		let main_sender = main_sender.clone();
		std::thread::spawn(move || {
			// create another inotify instance to watch the file
			let mut inotify = inotify::Inotify::init().expect("failed to create inotify instance");
			inotify.watches().add(&format!("./live/{file_name}"), inotify::WatchMask::MODIFY).expect("failed to watch file");

			let mut last_pos = 0;
			loop {
				// open the file
				let file = std::fs::File::open(&format!("./live/{}",file_name.clone()));
				if let Ok(file) = file {
					// read the file
					let mut reader = std::io::BufReader::new(file);

					// seek to the last position
					reader.seek(std::io::SeekFrom::Start(last_pos)).expect("failed to seek to the last position");
					for line in reader.by_ref().lines() {
						let line = line.expect("failed to read line");
						let donation = serde_json::from_str::<DonationTypes>(&line);
						donation.map_or_else(|_e| {
							println!("invalid json: {line:?}");
						}, |donation| {
							match main_sender.send(LiveDonationTypes::Donation(file_name.clone(), donation)) {
								Ok(_) => {},
								Err(e) => {
									println!("Error: {e}");
								}
							};
						});
					}

					// get the new position
					last_pos = reader.stream_position().expect("failed to get the new position");
				} else {
					break;
				}

				// block until the file is modified
				let mut buffer = [0u8; 4096];
				inotify.read_events_blocking(&mut buffer).expect("failed to read events");
			}
		});
	}
	let m_sender = main_sender.clone();
	std::thread::spawn(move|| {
		let listener = match LocalSocketListener::bind("@live_donations") {
			Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
				println!("Error address already in use: {}", e);
				return;
			},
			x => x.unwrap(),
		};
		let mut status:HashMap<String, i32> = HashMap::new();
		for connection in listener.incoming() {
			let mut connection = connection.unwrap();
			// Receive the message on the server
			let mut buffer = Vec::new();
			let bytes_read = connection.read_to_end(&mut buffer).unwrap();
			let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);
			//println!("Received message: {}", received_message);
			// first line is the stream name
			let mut lines = received_message.lines();
			let stream_name = lines.next().unwrap();

			let read_status = status.entry(stream_name.to_string()).or_insert(0);
			// lines after that are the donations
			let donations:Vec<_> = lines.skip(*read_status as usize).map(|line| {
				serde_json::from_str::<DonationTypes>(line).unwrap()
			}).collect();

			// print the donations
			println!("stream name: {stream_name}");
				
			for donation in donations {
				m_sender.send(LiveDonationTypes::Donation(stream_name.to_string(), donation)).unwrap();
				// increment the read_status
				*read_status += 1;
			};	
		}
	});
	let thread_server = server.clone();
	std::thread::spawn(move || {
		//receive all the messages from the main thread
		loop {
			match receiver.recv() {
				Ok(LiveDonationTypes::Donation(file_name, donation)) => {
					// add the donation to the live donations
					// first trim .donations.json from the file_name
					let file_name = file_name.trim_end_matches(".donations.json").to_string();
					// then add live- to the file_name
					let file_name = format!("live-{file_name}");
					thread_server.do_send(LiveUpdate { livestream_name: file_name.clone(), donation });
				},
				Ok(LiveDonationTypes::StreamEnd(file_name)) => {
					//println!("Stream ended: {}",file_name);
					// remove the file from the live messages
					thread_server.do_send(StreamEnd { livestream_name: file_name.clone() });
				},
				Err(e) => {
					println!("Error: {e}");
				}
			}
		}
	});
	//server.do_send(AddRoom { name: "general".to_string() });

	// spawn a thread to update the live messages
	//let reader_actor = LiveReaderActor { live_donations: live_messages.clone(), chat_server:server.clone() }.start();
	//let reader_actor = LiveReaderActor { chat_server:server.clone() }.start();

	// wrap the database pool in a container
	let db_container: Pool<ConnectionManager<PgConnection>> = pool.clone();

    HttpServer::new(move || {
        App::new()
			.app_data(web::Data::new(server.clone()))
			.app_data(web::Data::new(db_container.clone()))
            .service(index)
			.service(streams)
			.service(chat)
			.service(login)
			.service(login_page)
			.service(streamers)
			.service(ws_index)
			.service(fs::Files::new("/files", "./chats"))
			.service(fs::Files::new("/static", "./static"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
