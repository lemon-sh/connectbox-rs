use std::net::Ipv4Addr;

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
    /// Log out and close the shell
    Exit,

    /// Manage port forwards
    #[command(name = "pfw")]
    PortForwards {
        #[command(subcommand)]
        cmd: PortForwardsCommand,
    },
}

#[derive(Parser, Debug)]
pub(crate) enum PortForwardsCommand {
    /// List all port forwards
    Show,
    /// Add a port forward
    Add {
        /// LAN address of the host to forward the port to
        local_ip: Ipv4Addr,
        /// External port range
        range: String,
        /// Internal port range. If unspecified, the same as external
        int_range: Option<String>,
        /// TCP, UDP or both
        #[arg(short, default_value = "both")]
        protocol: String,
        /// Add this port forward in disabled state
        #[arg(short)]
        disable: bool,
    },
    /// Enable, disable or delete a port forward
    Edit {
        /// ID of the port. You can use `pfw show` to find it
        id: u32,
        /// Action to perform with the port. Can be either enable, disable or delete.
        action: String,
    },
}

pub(crate) fn shell_cmd() -> Command {
    ShellCommand::augment_subcommands(Command::new("").multicall(true))
}
