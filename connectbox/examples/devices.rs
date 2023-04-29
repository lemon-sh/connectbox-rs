use std::env;

use color_eyre::Result;
use connectbox::ConnectBox;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let mut args = env::args().skip(1);
    let ip = args.next().expect("no ip specified");
    let code = args.next().expect("no code specified");

    let connect_box = ConnectBox::new(ip)?;
    connect_box.login(&code).await?;

    let devices = connect_box.get_devices().await?;
    println!("{devices:?}");

    Ok(())
}
