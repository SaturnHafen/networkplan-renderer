use crate::parser;

#[derive(Clone)]
pub enum Item {
    FriendlyName(String),
    IPv4(String),
    IPv6(String),
    Port(u16, String, String),
    OS(String),
    MAC(String),
}

impl Item {
    pub fn value(&self) -> String {
        match self {
            Item::FriendlyName(name) => name.to_string(),
            Item::IPv4(ip) => format!("IPv4: {}", ip),
            Item::IPv6(ip) => format!("IPv6: {}", ip),
            Item::Port(port, protocol, application) => {
                format!("{}/{} {}", port, protocol, application)
            }
            Item::OS(name) => format!("OS: {}", name),
            Item::MAC(mac) => format!("MAC: {}", mac),
        }
    }
}

#[derive(Clone)]
pub struct Server {
    pub items: Vec<Item>,
}

impl Server {
    fn new() -> Server {
        Server { items: Vec::new() }
    }

    pub fn into_items(host: parser::Host) -> Server {
        let mut server = Server::new();

        for name in host.hostnames {
            server.items.push(Item::FriendlyName(name));
        }

        for addr in host.addresses {
            match addr.addr_type {
                parser::AddrType::IPv4 => {
                    server.items.push(Item::IPv4(addr.address));
                }
                parser::AddrType::IPv6 => {
                    server.items.push(Item::IPv6(addr.address));
                }
                parser::AddrType::MAC => {
                    server.items.push(Item::MAC(addr.address));
                }
            }
        }

        for port in host.ports {
            let service_name = match port.service {
                Some(service) => service.name.unwrap_or_else(|| "unknown".to_string()),
                None => "unknown".to_string(),
            };
            server
                .items
                .push(Item::Port(port.port, port.protocol, service_name));
        }

        if host.os.is_some() {
            server.items.push(Item::OS(host.os.unwrap()));
        }

        server
    }
}
