use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use std::net::SocketAddr;

const INDEX: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust Microservice</title>
</head>
<body>
    <h3>Rust Microservice Example</h3>
</body>
</html>
"#;

fn main() {
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);

    // Make a server from the builder
    let server = builder.serve(|| service_fn(microservice_handler));

    // Drop any errors for simplicity
    // Maps errors from current type to the drop function
    let server = server.map_err(drop);

    // Actually runs the server
    hyper::rt::run(server);
}

fn microservice_handler(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = Error> {
    // Match on request method and path
    match (req.method(), req.uri().path()) {
        // GET for root path: return simple html landing page
        (&Method::GET, "/") => future::ok(Response::new(INDEX.into())),
        // Unknown: 404
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            future::ok(response)
        }
    }
}
