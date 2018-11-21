extern crate failure;
use failure::Error;

extern crate forge;
use forge::server::run_server;

fn main() -> Result<(), Error> {
    run_server()
}