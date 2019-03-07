

use std::error::Error;
use std::io;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, PasswordInput, Select};

use wizard::ConfigData;
use wizard::Manager;

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

    Ok(data)
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


    let systemd_user = Select::with_theme(&theme)
        .with_prompt("Which user should the daemon run on?")
        .default(0)
        .item("Kakapo") // TODO: only show if previous step is ok
        .item("$USER") //TODO: env
        .item("sudo")
        .interact()?;

    Ok(data)
}

pub fn manage_domains(data: ConfigData, check_if_exists: bool) -> Result<ConfigData, Box<Error>> {
    Ok(data)
}