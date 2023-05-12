// This example shows you how to add, edit and remove port forwading entries.
// Usage: cargo run --example port_forwards -- <Connect Box IP> <login password> <local IP>

use std::{env, net::Ipv4Addr};

use color_eyre::Result;
use connectbox::{
    models::{PortForwardEntry, PortForwardProtocol},
    ConnectBox, PortForwardAction,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let mut args = env::args().skip(1);
    let ip = args.next().expect("no ip specified");
    let password = args.next().expect("no password specified");
    let local_ip: Ipv4Addr = args
        .next()
        .expect("no local ip specified")
        .parse()
        .expect("local ip is not a valid ipv4 address");

    // first, we create a new API client and log in to the router.
    let connect_box = ConnectBox::new(ip, password, true)?;
    connect_box.login().await?;

    // then, we remove all port forwarding entries for the local ip
    connect_box
        .edit_port_forwards(|v| {
            if v.local_ip == local_ip {
                PortForwardAction::Delete
            } else {
                PortForwardAction::Keep
            }
        })
        .await?;

    // then, we add a new port forward for the port 25565 (Minecraft server)
    let pf = PortForwardEntry {
        id: 0,
        local_ip,
        start_port: 25565,
        end_port: 25565,
        start_port_in: 25565,
        end_port_in: 25565,
        protocol: PortForwardProtocol::Both,
        enable: true,
    };
    connect_box.add_port_forward(&pf).await?;

    // lastly, we get the new port forwarding table and print it out
    let portforwards = connect_box.port_forwards().await?;
    println!("{portforwards:#?}");

    // and then we log out so that other users can log in to the web interface
    connect_box.logout().await?;

    Ok(())
}
