use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct LanUserTable {
    #[serde(rename = "Ethernet")]
    #[serde(deserialize_with = "unwrap_xml_list")]
    pub ethernet: Vec<ClientInfo>,
    #[serde(rename = "WIFI")]
    #[serde(deserialize_with = "unwrap_xml_list")]
    pub wifi: Vec<ClientInfo>,
    #[serde(rename = "totalClient")]
    pub total_clients: u32,
    #[serde(rename = "Customer")]
    pub customer: String,
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

#[derive(Deserialize)]
struct List<T> {
    #[serde(rename = "$value")]
    #[serde(default = "Vec::default")]
    elems: Vec<T>,
}

fn unwrap_xml_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>
{
    Ok(List::deserialize(deserializer)?.elems)
}
