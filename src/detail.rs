use crate::parser;
use crate::parser::Host;
use crate::parser::Metadata;

#[derive(Debug, Clone)]
pub struct Hostservice {
    pub ip: String,
    pub port: u16,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub service: String,
    pub product: String,
    pub version: Option<String>,
    pub extrainfo: Option<String>,
    pub hosts: Vec<Hostservice>,
}

#[derive(Debug, Clone)]
pub struct Tables {
    pub services: Vec<Service>,
}

impl Tables {
    pub fn new() -> Tables {
        Tables {
            services: Vec::new(),
        }
    }

    fn correct_table(service: &Service, portservice: &parser::Service) -> bool {
        service.service == portservice.name.clone().unwrap()
            && service.product == portservice.product.clone().unwrap()
            && service.version == portservice.version
            && service.extrainfo == portservice.extrainfo
    }

    fn add_hostservice(service: &mut Service, addresses: &Vec<parser::IpAddr>, port: u16) {
        for address in addresses {
            match address.addr_type {
                parser::AddrType::MAC => {}
                _ => service.hosts.push(Hostservice {
                    ip: address.address.clone(),
                    port,
                    metadata: Metadata::None,
                }),
            }
        }
    }

    pub fn add_host(&mut self, host: &Host) {
        println!(
            "ports: {:?}",
            host.ports
                .clone()
                .iter()
                .map(|p| p.port)
                .collect::<Vec<u16>>()
        );
        for port in host.ports.clone() {
            println!(
                "{}:{} ==> {:?}",
                host.addresses.first().unwrap().address,
                port.port,
                &port.service
            );
            if port.service.is_some() {
                let mut add_service = true;
                for service in self.services.iter_mut() {
                    if Tables::correct_table(&service, &port.service.clone().unwrap()) {
                        Tables::add_hostservice(service, &host.addresses, port.port);
                        add_service = false;
                        break;
                    }
                }
                if add_service {
                    let portservice = port.service.clone().unwrap();
                    let mut service = Service {
                        service: portservice.name.unwrap(),
                        product: portservice.product.unwrap(),
                        version: portservice.version,
                        extrainfo: portservice.extrainfo,
                        hosts: Vec::new(),
                    };
                    Tables::add_hostservice(&mut service, &host.addresses, port.port);
                    self.services.push(service);
                }
            }
        }
    }
}
