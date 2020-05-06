#[macro_use]
extern crate diesel;

use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand,
};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use failure::Error;

pub mod models;
pub mod schema;

const CMD_ADD: &str = "add";
const CMD_LIST: &str = "list";
const DB_DEFAULT_FILE: &str = "test.db";

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("database")
                .short("d")
                .long("db")
                .value_name("FILE")
                .help("Sets a file name of a SQLite database")
                .takes_value(true),
        )
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
        .subcommand(SubCommand::with_name(CMD_LIST).about("prints a list with users"))
        .get_matches();

    // Get db path
    let path = matches.value_of("database").unwrap_or(DB_DEFAULT_FILE);

    // Set up db conn pool
    let manager = ConnectionManager::<SqliteConnection>::new(path);
    let pool = r2d2::Pool::new(manager)?;

    // Execute commands
    match matches.subcommand() {
        (CMD_ADD, Some(matches)) => {
            let conn = pool.get()?;
            // Prep data
            let name = matches.value_of("NAME").unwrap();
            let email = matches.value_of("EMAIL").unwrap();
            let uuid = format!("{}", uuid::Uuid::new_v4());
            let new_user = models::NewUser {
                id: &uuid,
                name,
                email,
            };
            // Insert data
            diesel::insert_into(schema::users::table)
                .values(&new_user)
                .execute(&conn)?;
        }
        (CMD_LIST, _) => {
            use self::schema::users::dsl::*;
            let conn = pool.get()?;
            // Query
            let items = users.load::<models::User>(&conn)?;
            // List result
            for user in items {
                println!("{:?}", user);
            }
        }
        _ => {
            matches.usage();
        }
    }

    Ok(())
}
