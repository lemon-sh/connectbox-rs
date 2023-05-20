use std::vec;

use ascii_table::{Align::Right, AsciiTable};
use color_eyre::Result;
use color_print::cprintln;
use connectbox::{models::{PortForwardEntry, PortForwardProtocol}, PortForwardAction};
use once_cell::sync::OnceCell;

use crate::{cli::PortForwardsCommand, AppState};

static PORT_FORWARDING_TABLE: OnceCell<AsciiTable> = OnceCell::new();

fn init_port_forwarding_table() -> AsciiTable {
    let mut t = AsciiTable::default();
    t.column(0).set_header("ID").set_align(Right);
    t.column(1).set_header("Local IP");
    t.column(2).set_header("Port range");
    t.column(3).set_header("Int. port range");
    t.column(4).set_header("Protocol");
    t.column(5).set_header("Enabled");
    t
}

pub(crate) async fn run(cmd: PortForwardsCommand, state: &AppState) -> Result<()> {
    match cmd {
        PortForwardsCommand::Show => {
            cprintln!("<blue!>Retrieving the port forwarding table...");
            let port_forwards = state.connect_box.port_forwards().await?;
            let table_entries = port_forwards.entries.into_iter().map(|e| {
                let port_range = format!("{}-{}", e.start_port, e.end_port);
                let in_port_range = format!("{}-{}", e.start_port_in, e.end_port_in);
                vec![
                    e.id.to_string(),
                    e.local_ip.to_string(),
                    port_range,
                    in_port_range,
                    e.protocol.to_string(),
                    e.enable.to_string(),
                ]
            });
            let rendered_table = PORT_FORWARDING_TABLE
                .get_or_init(init_port_forwarding_table)
                .format(table_entries);
            cprintln!(
                "<black!>LAN IP: {}\nSubnet mask: {}\n</black!>{rendered_table}",
                port_forwards.lan_ip,
                port_forwards.subnet_mask
            );
        }
        PortForwardsCommand::Add {
            local_ip,
            range,
            int_range,
            protocol,
            disable,
        } => {
            let Some(protocol) = PortForwardProtocol::new(&protocol) else {
                cprintln!("<red!>Invalid protocol {protocol:?}");
                return Ok(())
            };
            let Some(range) = parse_port_range(&range) else {
                cprintln!("<red!>Invalid external range {range:?}");
                return Ok(())
            };
            let int_range = if let Some(r) = int_range {
                let Some(r) = parse_port_range(&r) else {
                    cprintln!("<red!>Invalid internal range {r:?}");
                return Ok(())
                };
                r
            } else {
                range
            };
            let port = PortForwardEntry {
                id: 0,
                local_ip,
                start_port: range.0,
                end_port: range.1,
                start_port_in: int_range.0,
                end_port_in: int_range.1,
                protocol,
                enable: !disable,
            };
            state.connect_box.add_port_forward(&port).await?;
            cprintln!("<green!>Done!");
        }
        PortForwardsCommand::Edit { id, mut action } => {
            action.make_ascii_lowercase();
            let action = match action.as_str() {
                "enable" => {
                    cprintln!("<blue!>Enabling port {id}...");
                    PortForwardAction::Enable
                }
                "disable" => {
                    cprintln!("<blue!>Disabling port {id}...");
                    PortForwardAction::Disable
                }
                "delete" => {
                    cprintln!("<blue!>Deleting port {id}...");
                    PortForwardAction::Delete
                }
                _ => {
                    cprintln!("<red!>Invalid action {action:?}");
                    return Ok(())
                }
            };
            let mut modified = false;
            state.connect_box.edit_port_forwards(|p| {
                if p.id == id {
                    modified = true;
                    action
                } else {
                    PortForwardAction::Keep
                }
            }).await?;
            if !modified {
                cprintln!("<red!>No port with id {id} exists");
            } else {
                cprintln!("<green!>Done!")
            }
        },
    }
    Ok(())
}

fn parse_port_range(s: &str) -> Option<(u16, u16)> {
    let (start, end) = s.split_once('-')?;
    Some((start.parse().ok()?, end.parse().ok()?))
}
