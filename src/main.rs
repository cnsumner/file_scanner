// suppress warnings for issue #50504 <https://github.com/rust-lang/rust/issues/50504>
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate clap;
extern crate config;

#[macro_use]
extern crate diesel;

extern crate uuid;

use clap::{Arg, App, SubCommand};

pub mod schema;
use ::schema as Schema;

pub mod mods;
use mods::util as Util;
use mods::models as Models;
use mods::sql as Sql;
use mods::capabilities as Capabilities;

fn main() {
    let matches = App::new("File Scanner")
                        .version("0.0.1")
                        .author("Luke Prince github.com/jlprince21")
                        .about("Assists in indexing a collection of files")
                        .arg(Arg::with_name("action")
                            .short("a")
                            .long("action")
                            .value_name("ACTION")
                            .help("Select which action you want the file scanner to perform")
                            .takes_value(true))
                        .subcommand(
                            SubCommand::with_name("tag")
                                .arg(Arg::with_name("id").required(true).max_values(1))
                                .arg(Arg::with_name("tags").last(true).required(true).min_values(1)),
                        )
                        .subcommand(
                            SubCommand::with_name("newtag")
                                .arg(Arg::with_name("tag").required(true).max_values(1))
                        )
                        .get_matches();

    let settings: Util::Settings = Util::get_settings();
    let connection = Sql::establish_connection(&settings.pg_connection_string);
    let action = matches.value_of("action").unwrap_or("none");

    if let Some(matches) = matches.subcommand_matches("newtag") {
        let tag = matches.value_of("tag").unwrap_or("none");
        println!("{}", tag);
        Capabilities::create_tag(&connection, tag);
        std::process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("tag") {
        //  Example: cargo run -- tag 123 -- summer beach vacation
        let listing_id = value_t!(matches.value_of("id"), String).unwrap_or_else(|e| e.exit()); // handy macro from clap
        let tags: Vec<_> = matches.values_of("tags").unwrap().collect();

        for tag in tags {
            Capabilities::tag_listing(&connection, &listing_id, tag);
        }

        println!("Tag(s) applied successfully!");
        std::process::exit(0);
    }

    match action {
        "duplicates" => {
            println!("Searching for duplicate files...");
            Capabilities::find_duplicates(&connection);
        },
        "hash" => {
            println!("Hashing...");
            Capabilities::start_hashing(&settings.directory_to_scan, &connection);
        },
        "orphans" => {
            println!("Removing orphans from database...");
            Capabilities::delete_missing_listings(&connection);
        }
        _ => {
            println!("No valid args provided, exiting.");
            std::process::exit(0);
        }
    };
}