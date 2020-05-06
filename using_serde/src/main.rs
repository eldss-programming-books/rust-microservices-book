use futures::{future, Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use rand::distributions::{Bernoulli, Normal, Uniform};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::net::SocketAddr;
use std::ops::Range;

/// Stores a random number.
#[derive(Serialize, Deserialize)]
struct RngResponse {
    value: f64,
}

/// Types of random number requests.
#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
}

fn microservice_handler(
    req: Request<Body>,
) -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/random") => {
            let body = req.into_body().concat2().map(|chunks| {
                let res = serde_json::from_slice::<RngRequest>(chunks.as_ref())
                    .map(handle_request)
                    .and_then(|resp| serde_json::to_string(&resp));
                match res {
                    Ok(body) => Response::new(body.into()),
                    Err(err) => Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(err.to_string().into())
                        .unwrap(),
                }
            });
            Box::new(body)
        }
        _ => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".into())
                .unwrap();
            Box::new(future::ok(resp))
        }
    }
}

fn handle_request(request: RngRequest) -> RngResponse {
    let mut rng = rand::thread_rng();
    let value = {
        match request {
            RngRequest::Uniform { range } => rng.sample(Uniform::from(range)) as f64,
            RngRequest::Normal { mean, std_dev } => rng.sample(Normal::new(mean, std_dev)) as f64,
            RngRequest::Bernoulli { p } => rng.sample(Bernoulli::new(p)) as i8 as f64,
        }
    };
    RngResponse { value }
}

fn main() {
    let localhost: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&localhost);
    let server = builder.serve(|| service_fn(microservice_handler));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}
