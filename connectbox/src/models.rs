use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LanUserTable {
    #[serde(rename = "Ethernet")]
    pub ethernet: ClientInfos,
    #[serde(rename = "WIFI")]
    pub wifi: ClientInfos,
    #[serde(rename = "totalClient")]
    pub total_clients: u32,
    #[serde(rename = "Customer")]
    pub customer: String,
}

#[derive(Deserialize, Debug)]
pub struct ClientInfos {
    pub clientinfo: Vec<ClientInfo>,
}

#[derive(Deserialize, Debug)]
pub struct ClientInfo {
    pub index: u32,
    pub interface: String,
    #[serde(rename = "interfaceid")]
    pub interface_id: u32,
    #[serde(rename = "IPv4Addr")]
    pub ipv4_addr: String,
    pub hostname: String,
    #[serde(rename = "MACAddr")]
    pub mac: String,
    #[serde(rename = "leaseTime")]
    pub lease_time: String,
    pub speed: u32,
}
