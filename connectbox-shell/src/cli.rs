use clap::{Command, Parser, Subcommand};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Args {
    /// Address of the modem
    pub address: String,

    /// Password used to log in to the modem. If not supplied, it will be asked interactively
    pub password: Option<String>,

    /// Log level, one of 'trace', 'debug', 'info', 'warn', 'error'
    #[arg(short, default_value = "warn")]
    pub log_level: Level,
}

#[derive(Parser, Debug)]
pub(crate) enum ShellCommand {
    Exit,
    #[command(name = "pfw")]
    PortForwards,
}

pub(crate) fn shell_cmd() -> Command {
    ShellCommand::augment_subcommands(Command::new("").multicall(true))
}
