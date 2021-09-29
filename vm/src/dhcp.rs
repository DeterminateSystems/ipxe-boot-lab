// Lifted entirely from dhcp4r's example, thank you

use dhcp4r::{bytes_u32, u32_bytes};

use std::collections::HashMap;
use std::net::{Ipv4Addr, UdpSocket};
use std::ops::Add;
use std::time::{Duration, Instant};

use dhcp4r::{options, packet, server};

// Server configuration
const SERVER_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 2, 1);
const IP_START: [u8; 4] = [10, 0, 2, 15];
const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
const DNS_IPS: [Ipv4Addr; 2] = [
    // Google DNS servers
    Ipv4Addr::new(8, 8, 8, 8),
    Ipv4Addr::new(4, 4, 4, 4),
];
const ROUTER_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 2, 2);
const LEASE_DURATION_SECS: u32 = 7200;
const LEASE_NUM: u32 = 100;

// Derived constants
const IP_START_NUM: u32 = bytes_u32!(IP_START);

pub fn main() {
    let socket = UdpSocket::bind("10.0.2.1:67").unwrap();
    socket.set_broadcast(true).unwrap();

    let ms = MyServer {
        leases: HashMap::new(),
        last_lease: 0,
        lease_duration: Duration::new(LEASE_DURATION_SECS as u64, 0),
    };

    server::Server::serve(socket, SERVER_IP, ms);
}

struct MyServer {
    leases: HashMap<Ipv4Addr, ([u8; 6], Instant)>,
    last_lease: u32,
    lease_duration: Duration,
}

impl server::Handler for MyServer {
    fn handle_request(&mut self, server: &server::Server, in_packet: packet::Packet) {
        match in_packet.message_type() {
            Ok(options::MessageType::Discover) => {
                println!("discover");
                // Prefer client's choice if available
                if let Some(options::DhcpOption::RequestedIpAddress(addr)) =
                    in_packet.option(options::REQUESTED_IP_ADDRESS)
                {
                    let addr = *addr;
                    if self.available(&in_packet.chaddr, &addr) {
                        reply(server, options::MessageType::Offer, in_packet, &addr);
                        return;
                    }
                }
                // Otherwise prefer existing (including expired if available)
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
                    reply(server, options::MessageType::Offer, in_packet, &ip);
                    return;
                }
                // Otherwise choose a free ip if available
                for _ in 0..LEASE_NUM {
                    self.last_lease = (self.last_lease + 1) % LEASE_NUM;
                    if self.available(
                        &in_packet.chaddr,
                        &((IP_START_NUM + &self.last_lease).into()),
                    ) {
                        reply(
                            server,
                            options::MessageType::Offer,
                            in_packet,
                            &((IP_START_NUM + &self.last_lease).into()),
                        );
                        break;
                    }
                }
            }

            Ok(options::MessageType::Request) => {
                println!("request");
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
                    return;
                }
                let req_ip = match in_packet.option(options::REQUESTED_IP_ADDRESS) {
                    Some(options::DhcpOption::RequestedIpAddress(x)) => *x,
                    _ => in_packet.ciaddr,
                };
                if !&self.available(&in_packet.chaddr, &req_ip) {
                    nak(server, in_packet, "Requested IP not available");
                    return;
                }
                self.leases.insert(
                    req_ip,
                    (in_packet.chaddr, Instant::now().add(self.lease_duration)),
                );
                reply(server, options::MessageType::Ack, in_packet, &req_ip);
            }

            Ok(options::MessageType::Release) | Ok(options::MessageType::Decline) => {
                println!("release / decline");
                // Ignore requests to alternative DHCP server
                if !server.for_this_server(&in_packet) {
                    return;
                }
                if let Some(ip) = self.current_lease(&in_packet.chaddr) {
                    self.leases.remove(&ip);
                }
            }

            // TODO - not necessary but support for dhcp4r::INFORM might be nice
            _ => {}
        }
    }
}

impl MyServer {
    fn available(&self, chaddr: &[u8; 6], addr: &Ipv4Addr) -> bool {
        let pos: u32 = (*addr).into();
        pos >= IP_START_NUM
            && pos < IP_START_NUM + LEASE_NUM
            && match self.leases.get(addr) {
                Some(x) => x.0 == *chaddr || Instant::now().gt(&x.1),
                None => true,
            }
    }

    fn current_lease(&self, chaddr: &[u8; 6]) -> Option<Ipv4Addr> {
        for (i, v) in &self.leases {
            if &v.0 == chaddr {
                return Some(*i);
            }
        }
        return None;
    }
}

fn reply(
    s: &server::Server,
    msg_type: options::MessageType,
    req_packet: packet::Packet,
    offer_ip: &Ipv4Addr,
) {
    let _ = s.reply(
        msg_type,
        vec![
            options::DhcpOption::IpAddressLeaseTime(LEASE_DURATION_SECS),
            options::DhcpOption::SubnetMask(SUBNET_MASK),
            options::DhcpOption::Router(vec![ROUTER_IP]),
            options::DhcpOption::DomainNameServer(DNS_IPS.to_vec()),
        ],
        *offer_ip,
        req_packet,
    );
}

fn nak(s: &server::Server, req_packet: packet::Packet, message: &str) {
    let _ = s.reply(
        options::MessageType::Nak,
        vec![options::DhcpOption::Message(message.to_string())],
        Ipv4Addr::new(0, 0, 0, 0),
        req_packet,
    );
}
