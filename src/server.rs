use failure::Error;

use ws::{listen, Handler, Sender, Message, CloseCode, Handshake, Response, Request};

use std::ffi::c_void;

static FORGE_PROTOCOL: &str = "forge";

#[repr(C)]
struct PacketCallback {
    userdata: *mut c_void,
    func: Option<unsafe extern "C" fn(*mut c_void, *const c_void)>
}

pub struct Server {
    pub out: Sender,
}

impl Server {
    fn process_packet(&mut self, packet: &Vec<u8>) {
        println!("Received {} byte packet", packet.len());
    }
}

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        if req.protocols().unwrap().iter().find(|proto| { proto == &&FORGE_PROTOCOL }).is_some() {
            let mut resp = Response::from_request(req)?;
            resp.set_protocol(FORGE_PROTOCOL);
            Ok(resp)
        }
        else
        {
            Err(ws::Error::new(ws::ErrorKind::Protocol, format!("'{}' protocol not found", FORGE_PROTOCOL)))
        }
    }

    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("Client connected!");
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Binary(packet) => {
                Ok(self.process_packet(&packet))
            },
            Message::Text(_) => Err(ws::Error::new(ws::ErrorKind::Protocol, "Text data not supported by protocol"))
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            CloseCode::Status => println!("The client closed the socket."),
            _ => println!("The client encountered an error: {:#?}, reason: {}", code, reason),
        }
        self.out.shutdown().unwrap();
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn run_server() -> Result<(), Error> {
  println!("Starting server");
  listen("127.0.0.1:8005", |out| Server { out: out } )?;
  Ok(())
}