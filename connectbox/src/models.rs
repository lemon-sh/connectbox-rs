use std::net::Ipv4Addr;
use std::time::Duration;

use serde::de::{self, Error, Unexpected};
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
    pub interface: String,
    #[serde(rename = "IPv4Addr")]
    pub ipv4_addr: String,
    pub index: u32,
    #[serde(rename = "interfaceid")]
    pub interface_id: u32,
    pub hostname: String,
    #[serde(rename = "MACAddr")]
    pub mac: String,
    pub method: u32,
    #[serde(rename = "leaseTime")]
    #[serde(deserialize_with = "deserialize_lease_time")]
    pub lease_time: Duration,
    pub speed: u32,
}

#[derive(Deserialize, Debug)]
pub struct PortForwards {
    #[serde(rename = "LanIP")]
    pub lan_ip: Ipv4Addr,
    #[serde(rename = "subnetmask")]
    pub subnet_mask: Ipv4Addr,
    #[serde(rename = "instance")]
    pub entries: Vec<PortForwardEntry>,
}

#[derive(Deserialize, Debug)]
pub struct PortForwardEntry {
    pub id: u32,
    #[serde(rename = "local_IP")]
    pub local_ip: Ipv4Addr,
    pub start_port: u16,
    pub end_port: u16,
    #[serde(rename = "start_portIn")]
    pub start_port_in: u16,
    #[serde(rename = "end_portIn")]
    pub end_port_in: u16,
    pub protocol: PortForwardProtocol,
    #[serde(deserialize_with = "bool_from_int")]
    pub enable: bool,
}

#[derive(Debug)]
pub enum PortForwardProtocol {
    Tcp,
    Udp,
    Both,
}

impl PortForwardProtocol {
    pub(crate) fn id_str(&self) -> &str {
        match self {
            PortForwardProtocol::Tcp => "1",
            PortForwardProtocol::Udp => "2",
            PortForwardProtocol::Both => "3",
        }
    }
}

impl<'de> Deserialize<'de> for PortForwardProtocol {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u8::deserialize(d)? {
            1 => Ok(Self::Tcp),
            2 => Ok(Self::Udp),
            3 => Ok(Self::Both),
            _ => Err(D::Error::custom("protocol not in range 1..=3")),
        }
    }
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(u64::from(other)),
            &"zero or one",
        )),
    }
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
    Ok(Duration::from_secs(u64::from(secs_total)))
}
