#![allow(deprecated)]
#![allow(unused_imports)]

//! This is the CLI implementation for the artid application, it allows the app to run from
//! the command line or to start as a GUI.
//!
//! The core of the application lives on the lib directory. This file and its modules job
//! is to parse the command line arguments and transform them into the proper calls to the
//! core.

#[macro_use]
extern crate clap;
extern crate chrono;
extern crate failure;
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure_derive;

extern crate artid as app; // Application library

use clap::App;
use failure::Fail;
use libc::EXIT_FAILURE;
use std::process::exit;

use app::logger;

mod errors;
mod parser;

use parser::Instance;

fn main() {
    if logger::init("info").is_err() {
        println!("Unable to start the logging implementation");
        exit(EXIT_FAILURE);
    }

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    let instance = Instance::new(&matches);
    if let Err(err) = instance.run() {
        if instance.backtrace() {
            err.causes().for_each(|cause| error!("{}", cause));
        } else {
            error!("{}", err);
        }

        exit(EXIT_FAILURE);
    }
}
