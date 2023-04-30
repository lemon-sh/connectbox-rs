use std::time::Duration;

use serde::de::Error;
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
    #[serde(deserialize_with = "deserialize_lease_time")]
    pub lease_time: Duration,
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
    T: Deserialize<'de>,
{
    Ok(List::deserialize(deserializer)?.elems)
}

fn deserialize_lease_time<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let mut fields = <&str as Deserialize>::deserialize(d)?.split(':').map(|f| {
        f.parse::<u32>()
            .map_err(|e| D::Error::custom(e.to_string()))
    });
    let days = fields
        .next()
        .ok_or(D::Error::custom("no days field in lease time"))??;
    let hours = fields
        .next()
        .ok_or(D::Error::custom("no hours field in lease time"))??;
    let mins = fields
        .next()
        .ok_or(D::Error::custom("no mins field in lease time"))??;
    let secs = fields
        .next()
        .ok_or(D::Error::custom("no secs field in lease time"))??;
    let secs_total = days * 86400 + hours * 3600 + mins * 60 + secs;
    Ok(Duration::from_secs(secs_total as u64))
}
