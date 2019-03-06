
use std::path::PathBuf;
use std::error::Error;

use ansi_term::Style;
use ansi_term::Color::{Green, Yellow, Red, RGB};
use inflector::Inflector;

mod steps;

use self::steps::get_theme;
use dialoguer::Confirmation;

pub enum Reason {
    NoConfigFile,
    InitialConfigure,
    ReconfigureAll(PathBuf),
    Reconfigure(String, PathBuf),
    ShowSteps,
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
    Nothing,
}

type StepFunction = Box<Fn() -> Result<(), Box<Error>>>;

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

fn start_configure_all(what: ConfigureWhat) {

    let steps = all_steps();

    let theme = get_theme();
    for (idx, step) in steps.iter().enumerate() {
        let (step_name, step_op) = step;
        let step_name_capitalized = step_name.to_title_case();


        match &what {
            ConfigureWhat::Nothing => {
                println!("{} {}",
                    RGB(131, 221, 2).bold().paint(&format!("({})", idx+1)),
                    step_name_capitalized);
            },
            ConfigureWhat::Everything => {
                println!("\n\t{} {}\n",
                    RGB(131, 221, 2).bold().paint(&format!("({})", idx+1)),
                    step_name_capitalized);

                //TODO: check if already exists
                let continue_step = Confirmation::with_theme(&theme)
                    .with_text("Continue?")
                    .interact()
                    .unwrap_or(false);

                if continue_step {
                    (step_op)();
                }
            },
            ConfigureWhat::One(ref step_to_configure) => {
                let step_to_configure_canonical = step_to_configure.to_lowercase().replace("_", " ");
                let step_idx = format!("{}", idx+1);
                let step_name = step_name.to_owned();

                if step_to_configure_canonical == step_idx || step_to_configure_canonical == step_name {
                    (step_op)();
                }
            },
        }
    }
}

pub fn get_possible_values() -> Vec<&'static str> {
    let steps = all_steps();

    steps
        .into_iter()
        .map(|x| x.0)
        .collect()
}

pub fn start(reason: Reason) {
    match reason {
        Reason::NoConfigFile => {
            print_welcome();
            println!("{}", Red.paint("    No Config file found, Starting the Configuration wizard"));
            start_configure_all(ConfigureWhat::Everything);
        },
        Reason::InitialConfigure => {
            print_welcome();
            start_configure_all(ConfigureWhat::Everything);
        },
        Reason::ReconfigureAll(config_file) => {
            print_welcome();
            start_configure_all(ConfigureWhat::Everything);
        },
        Reason::Reconfigure(step, config_file) => {
            start_configure_all(ConfigureWhat::One(step));
        },
        Reason::ShowSteps => {
            start_configure_all(ConfigureWhat::Nothing);
        },
    }
}