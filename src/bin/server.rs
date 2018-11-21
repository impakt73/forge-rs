use failure::Error;

use forge::server::run_server;

fn main() -> Result<(), Error> {
    run_server()
}