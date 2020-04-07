use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};
use dotenv::dotenv;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use log::{debug, info, trace, warn};
use serde_derive::Deserialize;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;

#[derive(Deserialize)]
struct Config {
    address: SocketAddr,
}

fn main() {
    // Enable use of .env file in this program
    dotenv().ok();
    // Start up the logger implementation
    pretty_env_logger::init();

    // Get command line args
    let matches = App::new("Server with keys")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("run").about("run the server").arg(
                Arg::with_name("address")
                    .short("a")
                    .long("address")
                    .value_name("ADDRESS")
                    .takes_value(true)
                    .help("address of the server"),
            ),
        )
        .subcommand(SubCommand::with_name("key").about("generates a secret key for cookies"))
        .get_matches();

    // Gets info from a toml config file
    let config = File::open("microservice.toml")
        .and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            Ok(buffer)
        })
        .and_then(|buffer| {
            toml::from_str::<Config>(&buffer)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
        })
        .map_err(|err| {
            warn!("Can't read config file: {}", err);
        })
        .ok();

    info!("Rand Microservice - v0.1.0");
    trace!("Starting...");

    // Get the address from an environment variable or default to localhost
    let localhost = ([127, 0, 0, 1], 8080);
    let addr: SocketAddr = matches
        // Prioritize cmd line args
        .value_of("address")
        .map(|s| s.to_string())
        // If none given, try env variable
        .or(env::var("ADDRESS").ok())
        .and_then(|addr| addr.parse().ok())
        // If none given, try config file
        .or(config.map(|config| config.address))
        // Default value
        .or_else(|| Some(localhost.into()))
        // At this point we are guaranteed a value
        .unwrap();

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

    // Use RUST_LOG env variable to see logs
    // ex: RUST_LOG=rand_value=trace,warn
    // this sets the log filter level to trace for all targets
    // (crates) with the 'rand_value' prefix and to warn for all
    // other targets. Can also use a .env file without the target
    // specification.
    debug!("Run!");
    hyper::rt::run(server);
}
