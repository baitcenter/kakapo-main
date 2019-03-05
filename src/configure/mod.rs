
use ansi_term::Style;
use ansi_term::Color::{Green, RGB};

mod steps;

pub enum Reason {
    ConfigureAll,
    InitialConfigure,
    Reconfigure(String),
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
   ++++++++++++++BBBBBB+++++++++++++++BBBBBB++++++++++++++
  +++++++++++BBBBBBBBBBBBBB+++++++BBBBBBBBBBBBBB+++++++++++
  +++++++++BBBBBBBBBBBBBBBBBB+++BBBBBBBBBBBBBBBBBB+++++++++
  +++++++BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB+++++++
  ++++++BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB++++++
 =======BBBBBBBBBB##BBBBBBBBBBBBBBBBBBBBB##BBBBBBBBBB=======
  =====BBBBBBBBBBB##BBBBBBBBBBBBBBBBBBBBB##BBBBBBBBBBB=====
  ======BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB======
  ======BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB======
  =======BBBBBBBBBBBBBBBBBBBBB*BBBBBBBBBBBBBBBBBBBBB=======
   ========BBBBBBBBBBBBBBBB*******BBBBBBBBBBBBBBBB========
   ===========BBBBBBBBBB*************BBBBBBBBBB===========
    =============BBBB*******************BBBB=============
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
      .replace('#', &format!("{}", Style::new().on(RGB(0, 0, 0)).paint(" ")))
      .replace('B', &format!("{}", Style::new().on(RGB(255, 255, 255)).paint(" ")))
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
    documentation=Green.paint("More information about this tool is available at https://example.com"));
}

fn start_configure_all() {
    let steps = [
        ("Create Central Database", steps::create_central_database),
    ];

    for (idx, step) in steps.iter().enumerate() {
        let (step_name, step_op) = step;
        println!("At step  {} {}", idx, step_name);

        (step_op)();
    }
}

pub fn start(reason: Reason) {
    match reason {
        Reason::ConfigureAll => {
            print_welcome();
            start_configure_all();
        },
        Reason::InitialConfigure => {
            print_welcome();
            //print error message saying that kakapo home not found
            start_configure_all();
        },
        Reason::Reconfigure(step) => {

        },
    }
}