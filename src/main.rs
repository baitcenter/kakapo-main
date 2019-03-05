#![allow(proc_macro_derive_resolution_fallback)]

/// Crates
extern crate ansi_term;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rpassword;

extern crate kakapo_api;

mod configure;

/// Extenal dependencies
use log::LevelFilter;
use env_logger::{Builder, Target};
use clap::{Arg, App};

use configure::Reason;

/// Internal dependencies
fn main() {

    let matches = App::new("Kakapo")
        .version("0.1.0")
        .author("Atta Z. <atta.h.zadeh@gmail.com>")
        .about("Database utility and Crud app creator")
        .arg(Arg::with_name("Verbosity")
            .short("v")
            .long("verbose")
            .help("Sets the level of verbosity"))
        .arg(Arg::with_name("Reconfigure")
            .long("reconfigure")
            .help("Set up the initial configuration again"))
        .arg(Arg::with_name("No Auth")
            .long("no-auth")
            .help("Do not authenticate user, [WARNING: don't use this in production]"))
        .get_matches();

    let do_configure = true;
    let reason = Reason::ConfigureAll;

    if do_configure {
        configure::start(reason);
    } else {
        //std::env::set_var("RUST_LOG", "warn,actix_web=info,kakapo=all");
        //std::env::set_var("RUST_BACKTRACE","1");
        Builder::new()
            .target(Target::Stdout)
            .filter_level(LevelFilter::Warn)
            .filter_module("kakapo", LevelFilter::Debug)
            .filter_module("actix_web", LevelFilter::Info)
            .init();

        kakapo_api::run();
    }
}