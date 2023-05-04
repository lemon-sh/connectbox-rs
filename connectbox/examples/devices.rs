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
    let code = args.next().expect("no code specified");

    let connect_box = ConnectBox::new(ip, code, true)?;
    connect_box.login().await?;

    connect_box
        .edit_port_forwards(|v| {
            if v.local_ip == Ipv4Addr::new(192, 168, 0, 180) {
                PortForwardAction::Delete
            } else {
                PortForwardAction::Keep
            }
        })
        .await?;

    let pf = PortForwardEntry {
        id: 0,
        local_ip: "192.168.0.180".parse().unwrap(),
        start_port: 25565,
        end_port: 25565,
        start_port_in: 25565,
        end_port_in: 25565,
        protocol: PortForwardProtocol::Both,
        enable: true,
    };
    connect_box.add_port_forward(&pf).await?;

    let portforwards = connect_box.port_forwards().await?;
    println!("{portforwards:#?}");

    connect_box.logout().await?;

    Ok(())
}
