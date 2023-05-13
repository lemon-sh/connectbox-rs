use color_print::{cstr, cprintln};

use clap::{FromArgMatches, Parser};
use cli::Args;
use color_eyre::Result;
use connectbox::ConnectBox;
use rustyline::{error::ReadlineError, DefaultEditor};

use crate::{cli::ShellCommand, utils::QuotableArgs};

mod cli;
mod utils;
mod commands;

pub(crate) struct AppState {
    connect_box: ConnectBox
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut shell_cmd = cli::shell_cmd();

    color_eyre::install()?;
    tracing_subscriber::fmt::fmt()
        .with_max_level(args.log_level)
        .init();

    let mut rl = DefaultEditor::new()?;
    let password = if let Some(password) = args.password {
        password
    } else {
        rl.readline("Password: ")?
    };
    let history_path = dirs::data_dir()
        .unwrap_or_default()
        .join(".connectbox-shell-history");
    let _err = rl.load_history(&history_path);

    cprintln!("<blue!>Logging in...");
    let connect_box = ConnectBox::new(args.address, password, true)?;
    connect_box.login().await?;
    let state = AppState { connect_box };

    loop {
        match rl.readline(cstr!("\n<green!> > ")) {
            Ok(line) => {
                if line.chars().all(char::is_whitespace) {
                    continue;
                }
                let cmd = match shell_cmd.try_get_matches_from_mut(QuotableArgs::new(&line)) {
                    Ok(mut matches) => ShellCommand::from_arg_matches_mut(&mut matches)?,
                    Err(e) => {
                        e.print()?;
                        continue;
                    }
                };
                match cmd {
                    ShellCommand::Exit => break,
                    ShellCommand::PortForwards { cmd } => commands::pfw::run(cmd, &state).await?,
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                println!("{err:?}");
                break;
            }
        }
    }
    println!("Logging out...");
    state.connect_box.logout().await?;

    rl.save_history(&history_path)?;
    Ok(())
}
