use anyhow::Result;
use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::{collections::HashMap, net::Ipv4Addr};

pub fn get_host_ipv4() -> Result<Ipv4Addr> {
    let ip = local_ip()?;
    match ip {
        std::net::IpAddr::V4(ipv4) => Ok(ipv4),
        _ => anyhow::bail!("No IPv4 address found"),
    }
}

pub fn start_mdns_service(port: u16) -> Result<()> {
    let mdns = ServiceDaemon::new()?;
    let host_ipv4 = get_host_ipv4()?;
    let host_name = hostname::get()?.to_string_lossy().to_string();

    let mut properties = HashMap::new();
    properties.insert("path".to_string(), "/".to_string());

    let service_info = ServiceInfo::new(
        "_pos._tcp.local.",
        "pos-backend",
        &format!("{}.local.", host_name),
        host_ipv4.to_string().as_str(),
        port,
        Some(properties),
    )?;

    mdns.register(service_info)?;
    Ok(())
}
