

use std::error::Error;
use std::io;
use std::process::Command;
use std::process::Stdio;
use std::str::from_utf8;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use std::collections::BTreeMap;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, PasswordInput, Select};
use ansi_term::Color::{Green, Yellow, Red, RGB};

use wizard::ConfigData;
use wizard::Manager;
use wizard::utils;
use wizard::DomainInfo;

use config::CONFIG_YAML;

pub fn get_theme() -> ColorfulTheme {
    ColorfulTheme {
        defaults_style: Style::new().dim(),
        error_style: Style::new().red(),
        indicator_style: Style::new().yellow().bold(),
        inactive_style: Style::new().dim(),
        active_style: Style::new(),
        yes_style: Style::new().green().dim(),
        no_style: Style::new().yellow().dim(),
        values_style: Style::new().yellow(),
    }
}

pub fn create_central_database(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    let theme = get_theme();

    if check_if_exists && data.manager.is_some() {
        let continue_step = Confirmation::with_theme(&theme)
            .with_text("Continue?")
            .interact()
            .unwrap_or(false);

        if !continue_step {
            return Ok(data)
        }
    }

    let use_postgres = Confirmation::with_theme(&theme)
        .with_text("Choose default database. Currently the only option is postgres. Do you want to continue?")
        .interact()?;

    if !use_postgres {
        return Err("Database type is not available".to_string().into());
    }

    let host: String = Input::with_theme(&theme)
        .with_prompt("Database host?")
        .default("127.0.0.1".to_string())
        .interact()?;

    let port: u16 = Input::with_theme(&theme)
        .with_prompt("Database port?")
        .default(5432)
        .interact()?;

    let user: String = Input::with_theme(&theme)
        .with_prompt("Database username?")
        .interact()?;

    let pass: String = PasswordInput::with_theme(&theme)
        .with_prompt("Database password?")
        .interact()?;

    let database: String = Input::with_theme(&theme)
        .with_prompt("Default database?")
        .default(user.to_owned())
        .interact()?;

    println!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, database);
    //TODO: test connection

    let mut new_data = data.to_owned();
    let manager = Manager {
        db_type: "postgres".to_string(),
        host, port,
        user, pass,
        database
    };

    new_data.manager = Some(manager);

    Ok(new_data)
}

pub fn setup_admin_account(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    let theme = get_theme();

    //TODO: check if user exists db::get_admin_user()

    let user: String = Input::with_theme(&theme)
        .with_prompt("Admin username?")
        .interact()?;

    let pass: String = PasswordInput::with_theme(&theme)
        .with_prompt("Admin password?")
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .interact()?;

    let email: String = Input::with_theme(&theme)
        .with_prompt("Admin Email?")
        .interact()?;

    let display_name: String = Input::with_theme(&theme)
        .with_prompt("Admin Name?")
        .default(user.to_owned())
        .interact()?;

    //TODO: build config data
    Ok(data)
}

pub fn setup_server(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    let theme = get_theme();
    /*
    if check_if_exists && data.server.is_some() {
        let continue_step = Confirmation::with_theme(&theme)
            .with_text("Continue?")
            .interact()
            .unwrap_or(false);

        if !continue_step {
            return Ok(data)
        }
    }
    */

    let server_name: String = Input::with_theme(&theme)
        .with_prompt("What is your server host (e.g. www.kakapo.io)")
        .interact()?;

    let default_port: u16 = Input::with_theme(&theme)
        .with_prompt("Which port to run the server on?")
        .default(1845)
        .interact()?;

    let tls = Select::with_theme(&theme)
        .with_prompt("Configure TLS")
        .default(0)
        .item("Setup with Let's Encrypt")
        .item("manual")
        .item("no")
        .interact()?;

    //TODO: set up tls if true

    Ok(data)
}

//TODO: linux only
pub fn create_kakapo_user(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    let theme = get_theme();

    let create_user = Confirmation::with_theme(&theme)
        .with_text("Create a user for the Kakapo process?")
        .interact()?;

    if !create_user {
        return Ok(data);
    }

    let result = Command::new("sudo")
        .arg("adduser")
        .arg("--system")
        .arg("--quiet")
        .arg("--group")
        .arg("kakapo")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    if result.status.success() {
        println!("{} successfully created", RGB(131, 221, 2).bold().paint("kakapo"));
    } else {
        return Err(Box::new(io::Error::new(io::ErrorKind::PermissionDenied, "Could not create user")));
    }

    let kakapo_passwd = Command::new("sudo")
        .arg("getent")
        .arg("passwd")
        .arg("kakapo")
        .output()?;

    let kakapo_passwd: Vec<&str> = from_utf8(&kakapo_passwd.stdout)?.split(":").collect();
    let kakapo_home = kakapo_passwd.get(5)
        .ok_or_else(|| Box::new(io::Error::new(io::ErrorKind::PermissionDenied, "Could not find kakapo home")))?;
    let mut kakapo_new_config_path = PathBuf::from(kakapo_home);
    kakapo_new_config_path.push(CONFIG_YAML);

    let new_kakapo_home = Select::with_theme(&theme)
        .with_prompt("Would you like to change the location of the config file?")
        .default(0)
        .item("Yes, put it in kakapo's home directory, and add a KAKPO_HOME variable to my bashrc file")
        .item("Yes, put it in kakapo's home directory, but keep my environment the same")
        .item("No, keep everything as is")
        .interact()?;

    let mut new_data = data.to_owned();
    match new_kakapo_home { //TODO: figure out the permission here, the user must have access to config.yaml...
        0 => {
            new_data.config_path = kakapo_new_config_path;
            let mut profile_path = env::home_dir()
                .ok_or_else(|| Box::new(io::Error::new(io::ErrorKind::PermissionDenied, "Could not find user home")))?;
            profile_path.push(".profile");
            env::set_var("KAKAPO_HOME", kakapo_home.to_owned());

            //TODO: try .zshrc, .bashrc, etc and prevent double appending to the file by being a little bit smarter
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(profile_path)?;

            writeln!(file, "export KAKAPO_HOME={}", &kakapo_home);
        },
        1 => {
            new_data.config_path = kakapo_new_config_path;
        },
        _ => {
            //nothing
        },
    }

    Ok(new_data)
}

//TODO: linux only
pub fn setup_daemon(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    let theme = get_theme();

    let setup_systemd = Confirmation::with_theme(&theme)
        .with_text("Set up a systemd service?")
        .interact()?;

    if !setup_systemd {
        return Ok(data);
    }

    let has_kakapo_user = Command::new("id")
        .arg("-u")
        .arg("kakapo")
        .output()?;

    let has_kakapo_user = has_kakapo_user
        .status
        .success();

    let current_user = Command::new("whoami")
        .output()?
        .stdout;
    let current_user = from_utf8(&current_user)?.trim();

    let systemd_user = if has_kakapo_user {
        Select::with_theme(&theme)
            .with_prompt("Which user should the daemon run as?")
            .default(0)
            .item("kakapo")
            .item(&current_user)
            .item("sudo")
            .interact()?
    } else {
        Select::with_theme(&theme)
            .with_prompt("Which user should the daemon run as?")
            .default(0)
            .item(&current_user) //TODO: check if current user is not sudo or kakapo
            .item("sudo")
            .interact()?
    };

    //TODO: After=network.target postgresql.service mysql.service redis.service rabbitmq-server.service ?
    let systemd_cfg = format!(r#"
[Unit]
Description=Kakapo system service
After=network.target
Wants=postgresql.service mysql.service redis.service rabbitmq-server.service

[Service]
Type=simple
User=kakapo
Group=kakapo
Environment="KAKAPO_HOME=stuff"
ExecStart=/usr/bin/boot.clock_fix start
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
"#);

    Ok(data)
}

pub fn manage_domains(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    fn get_add_or_remove_domains(theme: &ColorfulTheme) -> Result<&'static str, Box<Error>> {
        let action = Select::with_theme(theme)
            .with_prompt("Would you like to add or remove a domain?")
            .default(0)
            .item("Yes, add a new domain")
            .item("Yes, remove a domain")
            .item("No")
            .item("Let me see my domains")
            .interact()?;

        match action {
            0 => Ok("Yes, add a new domain"),
            1 => Ok("Yes, remove a domain"),
            2 => Ok("No"),
            _ => Ok("Let me see my domains")
        }
    }

    fn get_add_domains(theme: &ColorfulTheme) -> Result<&'static str, Box<Error>> {
        let action = Select::with_theme(theme)
            .with_prompt("Would you like to add or remove a domain?")
            .default(0)
            .item("Yes, add a new domain")
            .item("No")
            .interact()?;

        match action {
            0 => Ok("Yes, add a new domain"),
            _ => Ok("No"),
        }
    }

    let theme = get_theme();
    let mut new_data = data.to_owned();

    loop {
        let is_domains_empty = new_data.domains.is_empty();
        let action = if is_domains_empty {
            get_add_domains(&theme)?
        } else {
            get_add_or_remove_domains(&theme)?
        };

        match action {
            "Yes, add a new domain" => {
                let parrot_name = utils::random_parrot_name();
                let domain_name: String = Input::with_theme(&theme)
                    .with_prompt("Name of your new domain")
                    .default(parrot_name.to_owned())
                    .interact()?;

                let domain_type = Select::with_theme(&theme)
                    .with_prompt("What kind of domain is this?") //TODO: beta: find a way to get the types dynamically
                    .default(0)
                    .item("Postgres")
                    //.item("Redis") TODO: add the redis too
                    //.item("Local Python Runner") TODO: add the local script runner too
                    .interact()?;

                match domain_type {
                    _ => {
                        let host: String = Input::with_theme(&theme)
                            .with_prompt("Database host?")
                            .default("127.0.0.1".to_string())
                            .interact()?;

                        let port: u16 = Input::with_theme(&theme)
                            .with_prompt("Database port?")
                            .default(5432)
                            .interact()?;

                        let user: String = Input::with_theme(&theme)
                            .with_prompt("Database username?")
                            .interact()?;

                        let pass: String = PasswordInput::with_theme(&theme)
                            .with_prompt("Database password?")
                            .interact()?;

                        let database: String = Input::with_theme(&theme)
                            .with_prompt("Default database?")
                            .default(user.to_owned())
                            .interact()?;

                        let mut config_value = DomainInfo::Postgres {
                            host, port, user, pass, database,
                        };

                        new_data.domains.insert(domain_name, config_value);
                    }
                }
            },
            "Yes, remove a domain" => {
                let keys: Vec<_> = new_data.domains.keys().cloned().collect();
                let domain_to_remove = Select::with_theme(&theme)
                    .with_prompt("Which domain do you want to remove?") //TODO: beta: find a way to get the types dynamically
                    .default(0)
                    .items(&keys)
                    .interact()?;

                let domain_to_remove = keys.get(domain_to_remove)
                    .ok_or_else(|| Box::new(io::Error::new(io::ErrorKind::NotFound, "Out of range?")))?;

                let _ = new_data.domains.remove(domain_to_remove);

            },
            "Let me see my domains" => {
                println!("");
                for (domain_name, domain_data) in &new_data.domains {
                    println!("{}=> {}", RGB(131, 221, 2).bold().paint(format!("{: <16}", &domain_name)), &domain_data);
                }
            },
            _ => {
                return Ok(new_data);
            },
        };

        println!("");
    }
}