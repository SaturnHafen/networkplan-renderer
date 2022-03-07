use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;

use std::str;

#[derive(Debug, Clone)]
pub enum Metadata {
    SSH(String),
    None,
}

#[derive(Debug)]
enum ParserState {
    Ignore,
    WaitingForHost,
    Host,
    Hostnames,
    Ports,
    Port,
    Hops,
    Done,

    SSH,
    FINGERPRINT,
}

#[derive(Debug, Clone)]
pub struct Host {
    pub addresses: Vec<IpAddr>,
    pub hostnames: Vec<String>,
    pub ports: Vec<Port>,
    pub os: Option<String>,
    pub hops: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DuplicateKeys {
    pub addresses: Vec<IpAddr>,
    pub fingerprint: String,
}

#[derive(Debug, Clone)]
pub enum AddrType {
    IPv4,
    IPv6,
    MAC,
}

impl AddrType {
    fn parse(name: &[u8]) -> AddrType {
        match name {
            b"mac" => AddrType::MAC,
            b"ipv4" => AddrType::IPv4,
            b"ipv6" => AddrType::IPv6,
            _ => unreachable!("There should be no other AddrType!"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpAddr {
    pub address: String,
    pub addr_type: AddrType,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub protocol: String,
    pub port: u16,
    pub service: Option<Service>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub extrainfo: Option<String>,
}

pub struct Parser {
    state: ParserState,
    hosts: Vec<Host>,
    current_host: Host,
    current_port: Option<Port>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            state: ParserState::Ignore,
            hosts: Vec::new(),
            current_host: Host {
                addresses: Vec::new(),
                hostnames: Vec::new(),
                ports: Vec::new(),
                os: None,
                hops: Vec::new(),
            },
            current_port: None,
        }
    }

    fn process(&mut self, ev: Event) {
        // println!("Current State: {:?}, Event: {:?}", self.state, ev);

        self.state = match self.state {
            ParserState::Ignore => match ev {
                Event::Start(e) if e.local_name() == b"nmaprun" => ParserState::WaitingForHost,
                _ => ParserState::Ignore,
            },
            ParserState::WaitingForHost => match ev {
                Event::Start(e) if e.local_name() == b"host" => {
                    self.current_host = Host {
                        addresses: Vec::new(),
                        hostnames: Vec::new(),
                        ports: Vec::new(),
                        os: None,
                        hops: Vec::new(),
                    };
                    ParserState::Host
                }
                Event::End(e) if e.local_name() == b"nmaprun" => ParserState::Done,
                _ => ParserState::WaitingForHost,
            },
            ParserState::Host => match ev {
                Event::Empty(e) if e.local_name() == b"address" => {
                    // do filter magic
                    let addr = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == b"addr")
                        .unwrap()
                        .unwrap()
                        .value;

                    let addr_type = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == b"addrtype")
                        .unwrap()
                        .unwrap()
                        .value;

                    let addr_type = AddrType::parse(addr_type.as_ref());

                    self.current_host.addresses.push(IpAddr {
                        address: str::from_utf8(&addr).unwrap().to_string(),
                        addr_type,
                    });
                    ParserState::Host
                }
                Event::Start(e) if e.local_name() == b"hostnames" => ParserState::Hostnames,
                Event::Start(e) if e.local_name() == b"ports" => ParserState::Ports,
                Event::Start(e) if e.local_name() == b"trace" => ParserState::Hops,
                Event::End(e) if e.local_name() == b"host" => {
                    self.hosts.push(self.current_host.clone());
                    ParserState::WaitingForHost
                }
                _ => ParserState::Host,
            },
            ParserState::Hostnames => match ev {
                Event::Empty(e) if e.local_name() == b"hostname" => {
                    // do filter magic
                    let hostname = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == b"name")
                        .unwrap()
                        .unwrap()
                        .value;

                    self.current_host
                        .hostnames
                        .push(str::from_utf8(&hostname).unwrap().to_string());
                    ParserState::Hostnames
                }
                Event::End(e) if e.local_name() == b"hostnames" => ParserState::Host,
                _ => ParserState::Hostnames,
            },
            ParserState::Ports => match ev {
                Event::Start(e) if e.local_name() == b"port" => {
                    // do filter magic
                    let protocol = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == b"protocol")
                        .unwrap()
                        .unwrap()
                        .value;

                    let port = e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == b"portid")
                        .unwrap()
                        .unwrap()
                        .value;
                    let service = None;

                    let port = Port {
                        protocol: str::from_utf8(&protocol).unwrap().to_string(),
                        port: str::from_utf8(&port)
                            .unwrap()
                            .to_string()
                            .parse::<u16>()
                            .unwrap(),
                        service,
                        metadata: Metadata::None,
                    };

                    self.current_port = Some(port);

                    ParserState::Port
                }
                Event::End(e) if e.local_name() == b"ports" => ParserState::Host,
                _ => ParserState::Ports,
            },
            ParserState::Port => match ev {
                Event::Start(e) if e.local_name() == b"service" => {
                    // add service metadata
                    let mut service = Service {
                        name: None,
                        product: None,
                        version: None,
                        extrainfo: None,
                    };
                    for a in e.attributes() {
                        match a.as_ref().unwrap().key {
                            b"name" => {
                                service.name =
                                    Some(str::from_utf8(&a.unwrap().value).unwrap().to_string());
                            }
                            b"product" => {
                                service.product =
                                    Some(str::from_utf8(&a.unwrap().value).unwrap().to_string());
                            }
                            b"version" => {
                                service.version =
                                    Some(str::from_utf8(&a.unwrap().value).unwrap().to_string());
                            }
                            b"extrainfo" => {
                                service.extrainfo =
                                    Some(str::from_utf8(&a.unwrap().value).unwrap().to_string());
                            }
                            b"ostype" => {
                                self.current_host.os =
                                    Some(str::from_utf8(&a.unwrap().value).unwrap().to_string());
                            }
                            _ => {}
                        }
                    }

                    match &mut self.current_port {
                        Some(port) => port.service = Some(service).clone(),
                        None => unreachable!("There should be a port defined!"),
                    }
                    ParserState::Port
                }
                Event::End(e) if e.local_name() == b"port" => {
                    self.current_host
                        .ports
                        .push(self.current_port.clone().unwrap());
                    ParserState::Ports
                }
                _ => ParserState::Port,
            },
            ParserState::Hops => match ev {
                Event::Empty(e) if e.local_name() == b"hop" => {
                    // do filter magic
                    self.current_host.hops.push(
                        str::from_utf8(
                            &e.attributes()
                                .find(|a| a.as_ref().unwrap().key == b"ipaddr")
                                .unwrap()
                                .unwrap()
                                .value,
                        )
                        .unwrap()
                        .to_string(),
                    );

                    ParserState::Hops
                }
                Event::End(e) if e.local_name() == b"trace" => ParserState::Host,
                _ => ParserState::Hops,
            },
            ParserState::Done => ParserState::Done,

            ParserState::SSH => match ev {
                Event::Start(e) if e.local_name() == b"table" => ParserState::FINGERPRINT,

                _ => ParserState::SSH,
            },
            ParserState::FINGERPRINT => match ev {
                Event::Empty()
            }
        }
    }

    pub fn parse(filename: String) -> Vec<Host> {
        let file = File::open(filename).expect("Could not open file");
        let reader = BufReader::new(file);

        let mut xmlfile = Reader::from_reader(reader);

        let mut buf = Vec::new();

        let mut parser: Parser = Parser::new();

        loop {
            match xmlfile.read_event(&mut buf).unwrap() {
                Event::Eof => break,
                ev => parser.process(ev),
            }
        }
        parser.hosts
    }
}
