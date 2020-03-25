use dotenv::dotenv;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use log::{debug, info, trace};
use std::env;
use std::net::SocketAddr;

fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    info!("Rand Microservice - v0.1.0");
    trace!("Starting...");
    // Get the address from an environment variable or default to localhost
    let addr: SocketAddr = env::var("ADDRESS")
        .unwrap_or(String::from("127.0.0.1:8080"))
        .parse()
        .expect("can't parse ADDRESS variable");
    debug!("Trying to bind server to address: {:?}", addr);
    let builder = Server::bind(&addr);
    trace!("Creating service handler...");
    let server = builder.serve(|| {
        service_fn_ok(|req| {
            trace!("Incoming request is: {:?}", req);
            let random_byte: u8 = rand::random();
            trace!("Generated value is: {}", random_byte);
            Response::new(Body::from(random_byte.to_string()))
        })
    });
    info!("Used address: {}", server.local_addr());
    // Tell the server to drop any errors in the service function
    let server = server.map_err(drop);
    debug!("Run!");
    // Use RUST_LOG env variable to see logs
    // ex: RUST_LOG=rand_value=trace,warn
    // this sets the log filter level to trace for all targets
    // (crates) with the 'rand_value' prefix and to warn for all
    // other targets. Can also use a .env file without the target
    // specification.
    hyper::rt::run(server);
}
