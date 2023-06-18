use actix_web::{get, post, App, web, HttpResponse, HttpServer, Responder};
use askama_actix::{Template,TemplateToResponse};
use glob::glob;
use actix_files as fs;

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
}

#[get("/streams")]
async fn streamers() -> impl Responder {
	let mut out = Vec::new();
	// loop over chats directory
	let paths: Vec<_> = std::fs::read_dir("./chats").expect("Unable to read chat directory")
		.map(|r| r.unwrap())
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
	let template = ChannelsTemplate{channels: out};
	template.to_response()
}



#[get("/streams/{channel_name}")]
async fn streams(channel_name: web::Path<String>) -> impl Responder {
	let mut out = Vec::new();
	// loop over chats directory
	let paths: Vec<_> = std::fs::read_dir(format!("./chats/{channel_name}")).expect("Unable to read chat directory")
		.map(|r| r.unwrap())
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
		let thumbnail = glob(&format!("./chats/{channel_name}/{}/*.webp",folder_name)).expect("Failed to read thumbnail").next().expect("Failed to read thumbnail").expect("Failed to read thumbnail");
		let thumbnail_filename = thumbnail.as_path().file_name().expect("Failed to read thumbnail").to_str().expect("Failed to convert thumbnail to str").to_string();
		use urlencoding::encode;
		let thumbnail_filename = encode(&thumbnail_filename).into_owned();
		// remove file extension
		let stream_name = thumbnail.file_stem().expect("Failed to read file name").to_str().expect("Failed to convert file name to str").to_string();
		
		// get 
		let date_file_path = format!{"./chats/{channel_name}/{}/{}.date",folder_name,stream_name};
		let date_file = std::path::Path::new(&date_file_path);

		// parse date_file as i64
		// contents are just a number
		let date_released_string = std::fs::read_to_string(date_file).expect("Failed to read info file");

		/* 
		let desc_file_path = format!{"./chats/{}/{}.description",folder_name,stream_name};
		let desc_file = std::path::Path::new(&desc_file_path);
		let desc_file_name = desc_file.file_name().expect("failed to get filename").to_str().unwrap().to_string();
		*/
		let res = Stream{
			title: stream_name,
			thumbnail: format!{"/files/{channel_name}/{}/{}",folder_name,thumbnail_filename},
			link: format!("/chat/{channel_name}/{}",folder_name),
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
		format!("rgb({},{},{})",r,g,b)
	} else {
		format!("rgba({},{},{},{})",r,g,b,a)
	}
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Redemption {
	thumbnail_url: String,
	username: String,
	channel_id: String,
	sender: String,
	time: String,
	header_color: i64,
	body_color: i64,
}


#[derive(Serialize, Deserialize, Debug)]
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


#[derive(Serialize, Deserialize, Debug)]
struct Gift {
	username: String,
	channel_id: String,
	number: String,
	time: String,
	header_color: i64,
	body_color: i64,
	thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
	donations: Vec<DonationTypes>,
	colors: Vec<(i64,i64)>,
	video_id: String,
}

#[get("/chat/{channel_name}/{chat_name}")]
async fn chat(path: web::Path<(String,String)>) -> impl Responder {
	// print chat name
	
	let (channel_name,chat_name) = path.into_inner();
	//println!("chat name: {}",chat_name);

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
	for line in donations_file.lines() {
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
		donations.push(donation);
	}
	//println!("donations: {:?}",donations);
	// deduplicate colors
	donations_colors.sort();
	donations_colors.dedup();
	//println!("donations_colors: {:?}",donations_colors);
	let template = DonationsTemplate { donations: donations, colors: donations_colors, video_id: chat_name };
	template.to_response()
	

	// echo chat name
	//HttpResponse::Ok().body(format!("chat name: {}",chat_name))
}

use actix::prelude::*;
use actix_web::{Error, HttpRequest};
use actix_web_actors::ws;
use serde_json::{json, Value};

#[get("/ws/{chat_name}")]
async fn ws_index(r: HttpRequest, stream: web::Payload, path: web::Path<String>, server: web::Data<Addr<ChatServer>>) -> Result<HttpResponse, Error> {
	let chat_name = path.into_inner();
	let client_id = uuid::Uuid::new_v4().to_string();

	let actor = MyWebSocket { addr: server.get_ref().clone(), room: chat_name, is_user: false, hb: Instant::now(), username:String::new(), id: client_id };

    let resp = ws::start(actor, &r, stream);
    resp
}

use std::time::SystemTime;
use crate::schema::users::dsl::*;
#[post("/login")]
async fn login(body:web::Json<AuthenticationContents>, pool_wrap: web::Data<Pool<ConnectionManager<PgConnection>>> ) -> impl Responder {
	// get pool
	let mut pool = pool_wrap.get().unwrap();
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
	addr: Addr<MyWebSocket>,
}

impl Handler<GetDonations> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: GetDonations, _ctx: &mut Self::Context) -> Self::Result {

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
				msg.addr.do_send(ReadUpdate{ donation_id: id as i32, is_read: *status, video_id: msg.chat_name.clone(), username: String::new() });
			}
			return;
		}


		// get donations
		// do a database query 
		let mut pool = self.pool.get().unwrap();
		let videos = schema::video_donation_status::table
			.filter(schema::video_donation_status::id.eq(&msg.chat_name))
			.load::<models::VideoDonationStatus>(&mut pool)
			.expect("Error loading donations");

		// check if there are any donations
		let exists =  videos.len() > 0;

		if exists {
			for donations in videos {
				let list:Value = serde_json::from_str(donations.value.as_str()).expect("database corruption");
				let list = list.as_array().unwrap().into_iter().map(|x| x.as_bool().unwrap()).collect::<Vec<bool>>();
				
				// cache the results
				self.read_status.insert(msg.chat_name.clone(),list.clone());
				
				for (i,x) in list.iter().enumerate() {
					msg.addr.do_send(ReadUpdate{ donation_id: i as i32, is_read: *x, video_id: msg.chat_name.clone(), username: String::new() });
				}
			}
		} else {
			//println!("donations not found in database");

			// get donations file
			let donations_file = glob(&format!("./chats/*/{}/*.donations.json",msg.chat_name)).expect("Failed to read donations file").next().expect("Failed to read donations file").expect("Failed to read donations file");
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
				))
				.execute(&mut pool)
				.expect("Error saving new donations");

			// loop through donations
			for (i,v) in list.iter().enumerate() {
				// send update to client
				msg.addr.do_send(ReadUpdate{ donation_id: i as i32, is_read: *v, video_id: msg.chat_name.clone(), username: String::new() });
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
        println!("WebSocket started in room: {}",self.room);
		// add self to room
		self.addr.do_send(AddToRoom { chat_name: self.room.clone(), addr: ctx.address(), id: self.id.clone() });
		
		// check if database has information about this video
		// by calling the server
		self.addr.do_send(GetDonations { chat_name: self.room.clone(), addr: ctx.address()});
    }

	// called when websocket is stopped
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("WebSocket stopped");
		// remove self from room
		self.addr.do_send(RemoveFromRoom { chat_name: self.room.clone(), id: self.id.clone() });
    }
}

impl MyWebSocket {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
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
    addr: Addr<MyWebSocket>,
	id: String,
}
// Implement a signal handler for the server actor
impl Handler<AddToRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: AddToRoom, _ctx: &mut Self::Context) -> Self::Result {
        // Get the list of WebSocket actors for the specified room
        let room = self.rooms.entry(msg.chat_name.clone()).or_insert(HashMap::new());

        // Add the WebSocket actor to the room
        room.insert(msg.id,msg.addr);
    }
}

// Define a signal that the server can use to remove a WebSocket actor from a room
#[derive(Message)]
#[rtype(result = "()")]
struct RemoveFromRoom {
    chat_name: String,
    id: String,
}
// Implement a message handler for the server actor
impl Handler<RemoveFromRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RemoveFromRoom, _ctx: &mut Self::Context) -> Self::Result {
        // Get the list of WebSocket actors for the specified room
        if let Some(room) = self.rooms.get_mut(&msg.chat_name) {
			// remove the WebSocket actor from the hash map
			if room.remove(&msg.id).is_some() {
				println!("Removed {} from room {}",msg.id,msg.chat_name);
			}
        }
    }
}

use std::collections::HashMap;
// Define the server actor
struct ChatServer {
    rooms: HashMap<String, HashMap<String, Addr<MyWebSocket>>>,
	pool: Pool<ConnectionManager<PgConnection>>,
	read_status: HashMap<String, Vec<bool>>,
}

impl Actor for ChatServer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Server started");
    }
}

// Define a signal that the server can use to broadcast a message to all WebSocket actors in a room
#[derive(Message,Serialize,Deserialize,Debug)]
#[rtype(result = "()")]
struct ReadUpdate {
	donation_id:i32,
	is_read:bool,
	video_id:String,
	username:String,
}

// implement a message handler for the server actor
impl Handler<ReadUpdate> for ChatServer {
	type Result = ();

	fn handle(&mut self, msg: ReadUpdate, _ctx: &mut Self::Context) -> Self::Result {
		// get current read status
		if let Some(status) = self.read_status.get(&msg.video_id) {
			let mut pool = self.pool.get().unwrap();

			// change the status
			let mut status = status.clone();
			let previous_read_status = status[msg.donation_id as usize];
			status[msg.donation_id as usize] = msg.is_read;
			self.read_status.insert(msg.video_id.clone(), status.clone());
			

			// update other clients
			if let Some(room) = self.rooms.get_mut(&msg.video_id) {
				for addr in room.values() {
					addr.do_send(ReadUpdate { donation_id: msg.donation_id, is_read: msg.is_read, video_id: msg.video_id.clone(), username: String::new() });
				}
			} else {
				println!("room not found someow. this should not happen");
			}

			std::thread::spawn(move || {
			{ // own namespace to prevent overlap of id variable names
				use crate::schema::read_status_change_log::dsl::*;
	
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
						username.eq(&msg.username),
						video_id.eq(&msg.video_id),
						donation_id.eq(&msg.donation_id),
						previous_status.eq(&previous_read_status),
						new_status.eq(&msg.is_read),
						timestamp.eq(SystemTime::now()),
					))
					.get_result::<models::ReadStatusChangeLog>(&mut pool)
					.expect("Error inserting into read_status_change_log");	
			}
			{
				// update the database
				let serialized = serde_json::to_string(&status).unwrap();

				use crate::schema::video_donation_status::dsl::*;
				let _res = diesel::update(schema::video_donation_status::table)
					.filter(id.eq(&msg.video_id))
					.set( value.eq(serialized))
					.get_result::<models::VideoDonationStatus>(&mut pool)
					.expect("Error updating video_donation_status");
			}});
		} else {
			println!("client sent update for video not cached");
			return;
		};
	}
}

// same for client
impl Handler<ReadUpdate> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, msg: ReadUpdate, ctx: &mut Self::Context) -> Self::Result {
		//println!("ReadUpdate: {:?}",msg);
		// deserialize the message to ReadUpdateContents
		let contents = ReadUpdateContents { donation_id: msg.donation_id, is_read: msg.is_read };
		// serialize the message to json
		let json = serde_json::to_string(&contents).unwrap();
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
		// check if user exists in database
		let mut conn: PooledConnection<ConnectionManager<PgConnection>> = self.pool.get().unwrap();
		use schema::session_tokens::dsl::*;

		let results = session_tokens
			.filter(username.eq(&msg.username))
			.filter(token.eq(&msg.password))
			.load::<models::SessionToken>(&mut conn)
			.expect("Error loading session tokens");

		//println!("results: {:?}",results);

		if let Some(result) = results.first() {
			// send the message to the client
			msg.addr.do_send(AuthenticationToClient { exists: true, username: Some(result.username.clone()) })
		} else{
			// send the message to the client
			msg.addr.do_send(AuthenticationToClient { exists: false, username: None })
		}

	}
}

impl Handler<AuthenticationToClient> for MyWebSocket {
	type Result = ();

	fn handle(&mut self, msg: AuthenticationToClient, ctx: &mut Self::Context) -> Self::Result {
		self.is_user = msg.exists;
		if msg.exists {
			// set username
			self.username = msg.username.unwrap().clone();

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
					if self.is_user == false {
						ctx.text(json!({"status":"error","message":"not authenticated"}).to_string());
						return;
					}
					// send the message to the server
					self.addr.do_send(ReadUpdate { donation_id: contents.donation_id, is_read: contents.is_read, video_id: self.room.clone(), username: self.username.clone() });
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
				println!("Error: {:?}",e);
				ctx.stop();
			}
        }
    }
}

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	/*
	let vec = vec![true,false,true];
	let json_res = json!(vec);
	return Ok(()); */
	// create a pool of database connections
	dotenv::dotenv().ok();
    use std::env;
    let postgres_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let manager = ConnectionManager::<PgConnection>::new(postgres_url);
	let pool = Pool::builder().max_size(15).build(manager).unwrap();


    // Create the chat server instance
    let server = ChatServer { rooms: HashMap::new(), pool: pool.clone(), read_status: HashMap::new() }.start();

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
