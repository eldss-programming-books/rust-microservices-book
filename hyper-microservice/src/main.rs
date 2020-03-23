use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};
use std::net::SocketAddr;

fn main() {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);

    // Make a server from the builder
    let server = builder.serve(|| {
        // Service function
        // Ignores the request argument, gives same response to anything
        service_fn_ok(|_| Response::new(Body::from("Almost a microservice...But not there yet.")))
    });

    // Drop any errors for simplicity
    // Maps errors from current type to the drop function
    let server = server.map_err(drop);

    // Actually runs the server
    hyper::rt::run(server);
}
