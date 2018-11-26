use failure::Error;

use forge::server::run_server;

use std::ptr;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    let context = run_server("127.0.0.1", 8005, &forge::server::PacketCallback {userdata: ptr::null_mut(), func: None })?;
    thread::sleep(Duration::from_secs(3));
    context.shutdown()?;
    Ok(())
}