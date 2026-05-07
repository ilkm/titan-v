use std::net::Ipv4Addr;

/// Compute subnet broadcast address from interface IPv4 and netmask.
#[must_use]
pub fn ipv4_broadcast_from_mask(addr: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let a = u32::from_be_bytes(addr.octets());
    let m = u32::from_be_bytes(netmask.octets());
    Ipv4Addr::from(((a & m) | !m).to_be_bytes())
}

/// Check whether target IPv4 belongs to interface subnet.
#[must_use]
pub fn ipv4_in_subnet(target: Ipv4Addr, iface_ip: Ipv4Addr, netmask: Ipv4Addr) -> bool {
    let t = u32::from_be_bytes(target.octets());
    let i = u32::from_be_bytes(iface_ip.octets());
    let m = u32::from_be_bytes(netmask.octets());
    (t & m) == (i & m)
}
