pub mod id_generator;

use local_ip_address::local_ip;
use anyhow::Result;
use std::net::Ipv4Addr;

pub fn get_host_ipv4() -> Result<Ipv4Addr> {
    match local_ip()? {
        std::net::IpAddr::V4(ipv4) => Ok(ipv4),
        _ => Err(anyhow::anyhow!("No IPv4 address found")),
    }
}

pub fn start_mdns_service(_port: u16) -> Result<()> {
    // TODO: Implement mDNS service for discovery
    Ok(())
}
