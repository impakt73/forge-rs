use failure::Error;

use ws::{CloseCode, Handler, Handshake, Message, Request, Response, Sender};

use std::os::raw::c_void;
use std::ptr;
use std::thread;
use std::ffi::CStr;
use std::os::raw::c_char;

static FORGE_PROTOCOL: &str = "forge";

struct ForgeHandler {
    client_sender: Sender,
    callback: PacketCallback
}

impl ForgeHandler {
    fn process_packet(&mut self, packet: &Vec<u8>) {
        println!("Received {} byte packet", packet.len());
        unsafe {
            match self.callback.func {
                Some(func) => func(self.callback.userdata, std::mem::transmute(packet.as_ptr())),
                None => ()
            }
        }
    }
}

pub struct ServerContext {
    sender: Sender,
    thread: thread::JoinHandle<()>
}

impl ServerContext {
    pub fn shutdown(self) -> Result<(), Error> {
        println!("Server Shutting Down...");
        self.sender.shutdown()?;
        // @TODO: Maybe handle this differently?
        self.thread.join().unwrap();
        println!("Server Shut Down");
        Ok(())
    }
}

impl Handler for ForgeHandler {
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        if req
            .protocols()
            .unwrap()
            .iter()
            .find(|proto| proto == &&FORGE_PROTOCOL)
            .is_some()
        {
            let mut resp = Response::from_request(req)?;
            resp.set_protocol(FORGE_PROTOCOL);
            Ok(resp)
        } else {
            Err(ws::Error::new(
                ws::ErrorKind::Protocol,
                format!("'{}' protocol not found", FORGE_PROTOCOL),
            ))
        }
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("Client connected!");
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Binary(packet) => Ok(self.process_packet(&packet)),
            Message::Text(_) => Err(ws::Error::new(
                ws::ErrorKind::Protocol,
                "Text data not supported by protocol",
            )),
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            CloseCode::Status => println!("The client closed the socket."),
            _ => println!(
                "The client encountered an error: {:#?}, reason: {}",
                code, reason
            ),
        }
        self.client_sender.shutdown().unwrap();
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn run_server(host: &str, port: u32, callback: &PacketCallback) -> Result<ServerContext, Error> {
    println!("Server Starting...");

    let callback = callback.clone();
    let mut settings = ws::Settings::default();
    settings.max_connections = 1;
    let socket = ws::Builder::new()
        .with_settings(settings)
        .build(move |sender: ws::Sender| {
            ForgeHandler { client_sender: sender, callback: callback.clone() }
        }).unwrap();

    let server_socket = socket.broadcaster();

    let address = format!("{}:{}", host, port);

    let server_thread = thread::spawn(move || {
        println!("Server Listening on {}", address);
        socket.listen(address).unwrap();
    });

    Ok(ServerContext { sender: server_socket, thread: server_thread })
}

#[repr(C)]
#[derive(Clone)]
pub struct PacketCallback {
    pub userdata: *mut c_void,
    pub func: Option<unsafe extern "C" fn(*mut c_void, *const c_void)>,
}

// @TODO Is this correct...?
unsafe impl Send for PacketCallback {}
unsafe impl Sync for PacketCallback {}

#[repr(C)]
#[derive(Clone)]
pub struct ServerConfig {
    pub host: *const c_char,
    pub callback: PacketCallback,
    pub port: u32,
    _padding : u32
}

#[no_mangle]
pub unsafe extern "C" fn forge_start_server(config: *const ServerConfig) -> *mut ServerContext {
    if config.is_null() { return ptr::null_mut(); }

    let config = (*config).clone();
    if config.host.is_null() { return ptr::null_mut(); }
    if config.callback.func.is_none() { return ptr::null_mut(); }

    match run_server(&CStr::from_ptr(config.host).to_string_lossy(), config.port, &config.callback) {
        Ok(context) => {
            Box::into_raw(Box::new(context))
        },
        Err(_) => ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn forge_stop_server(context: *mut ServerContext) {
    if context.is_null() {
        let context = Box::from_raw(context);
        context.shutdown().unwrap();
    }
}