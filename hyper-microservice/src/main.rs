use futures::{future, Future};
use hyper::service::service_fn;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use regex::Regex;
use slab::Slab;
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

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

// Types used for a makeshift database of users
type UserId = u64;
// Empty for simplicity, would normally have fields and require serialization
// for db interaction/REST responses
struct UserData;
// Slab seems to be a mix of a hash map and a vector
type UserDb = Arc<Mutex<Slab<UserData>>>;

lazy_static! {
    static ref INDEX_PATH: Regex = Regex::new("^/(index\\.html?)?$").unwrap();
    static ref USER_PATH: Regex = Regex::new("^/user/((?P<user_id>\\d+?)/?)?$").unwrap();
    static ref USERS_PATH: Regex = Regex::new("^/users/?$").unwrap();
}

fn main() {
    // Set up server address
    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);

    // Setup user DB
    let user_db = Arc::new(Mutex::new(Slab::new()));

    // Make a server from the builder
    let server = builder.serve(move || {
        let user_db = user_db.clone();
        service_fn(move |req| microservice_handler(req, &user_db))
    });

    // Drop any errors for simplicity
    // Maps errors from current type to the drop function
    let server = server.map_err(drop);

    // Actually runs the server
    hyper::rt::run(server);
}

/// Provides functionality to handle HTTP requests to this server.
fn microservice_handler(
    req: Request<Body>,
    user_db: &UserDb,
) -> impl Future<Item = Response<Body>, Error = Error> {
    // Match on request method and path
    let response = {
        let method = req.method();
        let path = req.uri().path();
        // Lock mutex for full scope of response
        let mut users = user_db.lock().unwrap();

        // Root path: return simple html landing page
        if INDEX_PATH.is_match(path) {
            if method == &Method::GET {
                Response::new(INDEX.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }

        // All users path
        } else if USERS_PATH.is_match(path) {
            if method == &Method::GET {
                let list = users
                    .iter()
                    .map(|(id, _)| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                Response::new(list.into())
            } else {
                response_with_code(StatusCode::METHOD_NOT_ALLOWED)
            }

        // User REST Requests
        } else if let Some(cap) = USER_PATH.captures(path) {
            let user_id = cap.name("user_id").and_then(|m| {
                m.as_str()
                    .parse::<UserId>() // get the number as UserId
                    .ok() // convert the result to an option
                    .map(|x| x as usize) // convert to usize for Slab (ids are usize)
            });
            // Inner match on methods
            match (method, user_id) {
                /* POST */
                // Create new user and return id
                (&Method::POST, None) => {
                    let id = users.insert(UserData);
                    Response::new(id.to_string().into())
                }
                // Disallow client to give a user id
                (&Method::POST, Some(_)) => response_with_code(StatusCode::BAD_REQUEST),
                /* GET */
                // Get a user with a given id
                (&Method::GET, Some(id)) => {
                    if let Some(data) = users.get(id) {
                        Response::new(data.to_string().into())
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                /* PUT */
                // Update a user with a given id
                (&Method::PUT, Some(id)) => {
                    if let Some(user) = users.get_mut(id) {
                        // Access and replace
                        *user = UserData; // would have data in a real server
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                /* DELETE */
                // Remove a selected user
                (&Method::DELETE, Some(id)) => {
                    if users.contains(id) {
                        users.remove(id);
                        response_with_code(StatusCode::OK)
                    } else {
                        response_with_code(StatusCode::NOT_FOUND)
                    }
                }
                /* Default */
                _ => response_with_code(StatusCode::METHOD_NOT_ALLOWED),
            }

        // Nothing else matched: 404
        } else {
            response_with_code(StatusCode::NOT_FOUND)
        }
    }; // end response block
    future::ok(response)
}

/// Creates simple HTTP responses with the given status code.
fn response_with_code(status_code: StatusCode) -> Response<Body> {
    Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap()
}

/// Allow UserData to be represented as a String
impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Since the struct is empty, we don't show anything
        f.write_str("{}")
    }
}
