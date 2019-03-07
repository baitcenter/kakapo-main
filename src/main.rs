#![allow(proc_macro_derive_resolution_fallback)]

/// Crates
extern crate ansi_term;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rpassword;
extern crate console;
extern crate dialoguer;
extern crate inflector;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_yaml;

extern crate kakapo_api;

mod wizard;
mod config;

use std::path::PathBuf;
use std::fs;

use ansi_term::Color::Red;
use log::LevelFilter;
use env_logger::{Builder, Target};
use clap::{Arg, App, SubCommand};

use wizard::Reason;


fn main() {
    let matches = App::new("Kakapo")
        .version("0.1.0")
        .author("Atta Z. <atta.h.zadeh@gmail.com>")
        .about("Database utility and Crud app creator")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("path to config file"))
        .arg(Arg::with_name("verbosity")
            .short("v")
            .long("verbose")
            .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("configure")
            .about("Kakapo configuration wizard")
            .version("0.1.0")
            .author("Atta Z. <atta.h.zadeh@gmail.com>")
            .arg(Arg::with_name("step")
                .long("step")
                .short("s")
                .value_name("STEP")
                .required_unless("all")
                .conflicts_with("all")
                .takes_value(true)
                .possible_values(&wizard::get_possible_values())
                .help("Reconfigure step"))
            .arg(Arg::with_name("all")
                .long("all")
                .short("a")
                .help("Reconfigure everything")))
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(config) => Ok(PathBuf::from(config)),
        None => config::get_config_path(),
    };

    let config_file = match config_file {
        Ok(x) => x,
        Err(err) => {
            println!("{}", Red.bold().paint(err));
            return;
        },
    };

    let configuration_reason = if let Some(configure_matches) = matches.subcommand_matches("configure") {
        if !config_file.exists() {
            Some(Reason::InitialConfigure)
        } else if let Some(step) = configure_matches.value_of("step") {
            Some(Reason::Reconfigure(step.to_string(), config_file.to_owned()))
        } else {
            Some(Reason::ReconfigureAll(config_file.to_owned()))
        }
    } else {
        if !config_file.exists() {
            Some(Reason::NoConfigFile)
        } else {
            None
        }
    };

    if let Some(reason) = configuration_reason {
        wizard::start(reason, config_file);
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