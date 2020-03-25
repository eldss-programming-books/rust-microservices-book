use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};

fn main() {
    let addr = ([127, 0, 0, 1], 8080);
    let builder = Server::bind(&addr.into());
    let server = builder.serve(|| {
        service_fn_ok(|_| {
            let random_byte: u8 = rand::random();
            Response::new(Body::from(random_byte.to_string()))
        })
    });
    // Tell the server to drop any errors in the service function
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
