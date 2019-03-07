
use std::path::PathBuf;
use std::error::Error;
use std::fs;

use ansi_term::Style;
use ansi_term::Color::{Green, Yellow, Red, RGB};
use inflector::Inflector;

mod steps;
mod data;

use self::steps::get_theme;
use dialoguer::Confirmation;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Reason {
    NoConfigFile,
    InitialConfigure,
    ReconfigureAll(PathBuf),
    Reconfigure(String, PathBuf),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Version {
    V1,
}
impl Default for Version {
    fn default() -> Self { Version::V1 }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Manager {
    #[serde(rename = "type")]
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigData {
    pub version: Version,
    pub manager: Option<Manager>,
}

impl ConfigData {

    fn to_file(&self, path: PathBuf) -> Result<(), String> {
        let data = serde_yaml::to_string(self)
            .map_err(|err| err.to_string())?;

        fs::write(path, data)
            .map_err(|err| err.to_string())
    }

    fn from_file(path: PathBuf) -> Result<Self, String> {
        let config_data_str = fs::read_to_string(path)
            .map_err(|err| err.to_string())?;

        let data: ConfigData = serde_yaml::from_str(&config_data_str)
            .map_err(|err| err.to_string())?;

        Ok(data)
    }
}

fn print_welcome() {
    let output = r#"
                            ---
                  -----------------------
              -------------------------------
            -------------------------------------
          -----------------------------------------
        +++++++++++++++++++++++++++++++++++++++++++++
      +++++++++++++++++++++++++++++++++++++++++++++++++
     +++++++++++++++++++++++++++++++++++++++++++++++++++
    +++++++++++++++++++++++++++++++++++++++++++++++++++++
   +++++++++++++++++++++++++++++++++++++++++++++++++++++++
   ++++++++++++++######+++++++++++++++######++++++++++++++
  +++++++++++##############+++++++##############+++++++++++
  +++++++++##################+++##################+++++++++
  +++++++###########################################+++++++
  ++++++#############################################++++++
 =======##########..#####################..##########=======
  =====###########..#####################..###########=====
  ======#############################################======
  ======#############################################======
  =======#####################*#####################=======
   ========################*******################========
   ===========##########*************##########===========
    =============####*******************####=============
     ================*******************================
      %%%%%%%%%%%%%%%*******************%%%%%%%%%%%%%%%
        %%%%%%%%%%%%%*******************%%%%%%%%%%%%%
          %%%%%%%%%%%*******************%%%%%%%%%%%
            %%%%%%%%%*******************%%%%%%%%%
              %%%%%%%*******************%%%%%%
                    %*******************%
                       ***************
                         ***********
                             ***
    "#;

    let output = output
      .replace('-', &format!("{}", Style::new().on(RGB(0, 83, 34)).paint(" ")))
      .replace('.', &format!("{}", Style::new().on(RGB(0, 0, 0)).paint(" ")))
      .replace('#', &format!("{}", Style::new().on(RGB(255, 255, 255)).paint(" ")))
      .replace('+', &format!("{}", Style::new().on(RGB(14, 98, 31)).paint(" ")))
      .replace('=', &format!("{}", Style::new().on(RGB(86, 174, 13)).paint(" ")))
      .replace('%', &format!("{}", Style::new().on(RGB(131, 221, 2)).paint(" ")))
      .replace('*', &format!("{}", Style::new().on(RGB(243, 198, 26)).paint(" ")));

    println!(r#"
{logo}
             {title}
  {documentation}
    "#,
    logo=output,
    title=Style::new().bold().paint("WELCOME TO KAKAPO CONFIGURATION WIZARD"),
    documentation=Green.paint("More information about this tool is available at https://kakapo.io"));
}

enum ConfigureWhat {
    Everything,
    One(String),
}

type StepFunction = Box<Fn(ConfigData, bool) -> Result<ConfigData, Box<Error>>>;

fn all_steps() -> Vec<(&'static str, StepFunction)> {
     vec![
        ("create central database", Box::new(steps::create_central_database)),
        ("setup admin account", Box::new(steps::setup_admin_account)),
        ("setup server", Box::new(steps::setup_server)),
        ("create kakapo user", Box::new(steps::create_kakapo_user)), //linux only
        ("setup daemon", Box::new(steps::setup_daemon)), //linux only
        ("manage domains", Box::new(steps::manage_domains)),
    ]
}

fn start_configure_all(what: ConfigureWhat, config_data: ConfigData) -> Result<ConfigData, String> {

    let steps = all_steps();
    let mut config = config_data;

    for (idx, step) in steps.iter().enumerate() {
        let (step_name, step_op) = step;
        let step_name_capitalized = step_name.to_title_case();


        match &what {
            ConfigureWhat::Everything => {
                println!("\n\t{} {}\n",
                    RGB(131, 221, 2).bold().paint(&format!("({})", idx+1)),
                    step_name_capitalized);

                config = match (step_op)(config, true) {
                    Ok(config) => config,
                    Err(err) => {
                        return Err(err.to_string());
                    }
                };
                println!("new config: {:?}", &config);
            },
            ConfigureWhat::One(ref step_to_configure) => {
                let step_to_configure_canonical = step_to_configure.to_lowercase().replace("_", " ");
                let step_idx = format!("{}", idx+1);
                let step_name = step_name.to_owned();

                if step_to_configure_canonical == step_idx || step_to_configure_canonical == step_name {
                    config = match (step_op)(config, false) {
                        Ok(config) => config,
                        Err(err) => {
                            return Err(err.to_string());
                        }
                    };
                }
            },
        }
    }

    Ok(config)
}

pub fn get_possible_values() -> Vec<&'static str> {
    let steps = all_steps();

    steps
        .into_iter()
        .map(|x| x.0)
        .collect()
}

pub fn start_internal(reason: Reason) -> Result<ConfigData, String> {
    match reason {
        Reason::NoConfigFile => {
            let config_data = ConfigData::default();
            print_welcome();
            println!("{}", Red.paint("    No Config file found, Starting the Configuration wizard"));
            start_configure_all(ConfigureWhat::Everything, config_data)
        },
        Reason::InitialConfigure => {
            let config_data = ConfigData::default();
            print_welcome();
            start_configure_all(ConfigureWhat::Everything, config_data)
        },
        Reason::ReconfigureAll(config_file) => {
            let config_data = ConfigData::from_file(config_file)?;
            print_welcome();
            start_configure_all(ConfigureWhat::Everything, config_data)
        },
        Reason::Reconfigure(step, config_file) => {
            let config_data = ConfigData::from_file(config_file)?;
            start_configure_all(ConfigureWhat::One(step), config_data)
        },
    }
}

pub fn start(reason: Reason, out_file: PathBuf) {
    match start_internal(reason) {
        Ok(data) => {
            data.to_file(out_file);
        },
        Err(err) => {
            println!("{}", Red.bold().paint(err));
        }
    }
}