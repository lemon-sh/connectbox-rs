use std::{vec, fmt::Display};

use ascii_table::{AsciiTable, Align::Right};
use color_eyre::Result;
use color_print::cprintln;
use once_cell::sync::OnceCell;

use crate::{cli::PortForwardsCommand, AppState};

static PORT_FORWARDING_TABLE: OnceCell<AsciiTable> = OnceCell::new();

fn init_port_forwarding_table() -> AsciiTable {
    let mut t = AsciiTable::default();
    t.column(0).set_header("ID").set_align(Right);
    t.column(1).set_header("Local IP");
    t.column(2).set_header("Start port");
    t.column(3).set_header("End port");
    t.column(4).set_header("In. start port");
    t.column(5).set_header("In. end port");
    t.column(6).set_header("Protocol");
    t.column(7).set_header("Enabled");
    t
}

pub(crate) async fn run(cmd: PortForwardsCommand, state: &AppState) -> Result<()> {
    match cmd {
        PortForwardsCommand::Show => {
            cprintln!("<blue!>Retrieving the port forwarding table...");
            let port_forwards = state.connect_box.port_forwards().await?;
            let table_entries = port_forwards.entries.iter().map(|e| {
                let v: Vec<&dyn Display> = vec![&e.id, &e.local_ip, &e.start_port, &e.end_port, &e.start_port_in, &e.end_port_in, &e.protocol, &e.enable];
                v
            });
            let rendered_table = PORT_FORWARDING_TABLE.get_or_init(init_port_forwarding_table).format(table_entries);
            cprintln!("<black!>LAN IP: {}\nSubnet mask: {}\n</black!>{rendered_table}", port_forwards.lan_ip, port_forwards.subnet_mask);
        },
    }
    Ok(())
}