use crate::{cli::ShellCommand, utils::QuotableArgs};
use clap::{FromArgMatches, Parser};
use cli::Args;
use color_eyre::Result;
use color_print::{cformat, cprintln, cstr};
use connectbox::ConnectBox;
use rustyline::{
    error::ReadlineError,
    highlight::Highlighter,
    history::{DefaultHistory, FileHistory, MemHistory},
    Completer, DefaultEditor, Editor, Helper, Hinter, Validator,
};
use std::borrow::Cow;

mod cli;
mod commands;
mod utils;

pub(crate) struct AppState {
    connect_box: ConnectBox,
}

#[derive(Completer, Helper, Hinter, Validator)]
struct GreenPrompt;

impl Highlighter for GreenPrompt {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        cformat!("<green!>{prompt}").into()
    }
}

#[derive(Completer, Helper, Hinter, Validator)]
struct PasswordPrompt;

impl Highlighter for PasswordPrompt {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        cformat!("<red!>{prompt}").into()
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        "*".repeat(line.len()).into()
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut shell_cmd = cli::shell_cmd();

    color_eyre::install()?;
    tracing_subscriber::fmt::fmt()
        .with_max_level(args.log_level)
        .init();

    let mut rl = Editor::new()?;
    rl.set_helper(Some(GreenPrompt));

    let password = if let Some(password) = args.password {
        password
    } else {
        let mut rl = Editor::new()?;
        rl.set_helper(Some(PasswordPrompt));
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
        match rl.readline("\n >> ") {
            Ok(line) => {
                if line.chars().all(char::is_whitespace) {
                    continue;
                }
                let matches = shell_cmd.try_get_matches_from_mut(QuotableArgs::new(&line));
                rl.add_history_entry(line)?;
                let cmd = match matches {
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
    cprintln!("<blue!>Logging out...");
    state.connect_box.logout().await?;

    rl.save_history(&history_path)?;
    Ok(())
}
