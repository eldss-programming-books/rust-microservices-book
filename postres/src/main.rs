use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use postgres::{Client, Error, NoTls};
use r2d2_postgres::PostgresConnectionManager;
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::io;

// subcommands
const CMD_CREATE: &str = "create";
const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const CMD_IMPORT: &str = "import";

const DEFAULT_CONN: &str = "postgres://postgres:password@localhost:5432";

fn main() -> Result<(), failure::Error> {
    // Define commandline args/commands
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("db")
                .value_name("ADDR")
                .help("Sets an address of db connection")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name(CMD_CREATE).about("create users table"))
        .subcommand(
            SubCommand::with_name(CMD_ADD)
                .about("add user to the table")
                .arg(
                    Arg::with_name("NAME")
                        .help("Sets the name of a user")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("EMAIL")
                        .help("Sets the email of a user")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(SubCommand::with_name(CMD_LIST).about("list users in the table"))
        .subcommand(SubCommand::with_name(CMD_IMPORT).about("import users from csv"))
        .get_matches();

    // Get db connection string
    let addr = matches.value_of("database").unwrap_or(DEFAULT_CONN);

    // Establish connection pool
    let manager = PostgresConnectionManager::new(addr.parse().unwrap(), NoTls);
    let pool = r2d2::Pool::new(manager)?;
    let mut conn = pool.get()?;

    // Execute actions
    match matches.subcommand() {
        (CMD_CREATE, _) => {
            create_table(&mut conn)?;
        }
        (CMD_ADD, Some(matches)) => {
            let user = User {
                name: matches.value_of("NAME").unwrap().to_owned(),
                email: matches.value_of("EMAIL").unwrap().to_owned(),
            };
            create_user(&mut conn, &user)?;
        }
        (CMD_LIST, _) => {
            let list = list_users(&mut conn)?;
            for User { name, email } in list {
                println!("Name: {:20} Email: {:20}", name, email);
            }
        }
        (CMD_IMPORT, _) => {
            let mut rdr = csv::Reader::from_reader(io::stdin());
            let mut users: Vec<User> = Vec::new();
            for user in rdr.deserialize() {
                users.push(user?);
            }
            users
                .par_iter()
                .map(|user| -> Result<(), failure::Error> {
                    let mut conn = pool.get()?;
                    create_user(&mut conn, &user)?;
                    Ok(())
                })
                .for_each(drop);
        }
        _ => {
            matches.usage();
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct User {
    name: String,
    email: String,
}

fn create_table(conn: &mut Client) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )",
        &[],
    )
    .map(drop)
}

fn create_user(conn: &mut Client, user: &User) -> Result<(), Error> {
    conn.execute(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &[&user.name, &user.email],
    )
    .map(drop)
}

fn list_users(conn: &mut Client) -> Result<Vec<User>, Error> {
    let res = conn
        .query(
            "SELECT name, email
             FROM users",
            &[],
        )?
        .into_iter()
        .map(|row| User {
            name: row.get("name"),
            email: row.get("email"),
        })
        .collect();
    Ok(res)
}
