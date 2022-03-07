use crate::detail;
use crate::server;
use std::fs::File;
use std::io::Write;

pub static SERVER_ENTRY_HEIGHT: u64 = 20;
pub static SERVER_ENTRY_WIDTH: u64 = 150;
pub static IP_ENTRY_WIDTH: u64 = 100;
pub static PORT_ENTRY_WIDTH: u64 = 50;
static SERVER_PADDING: u64 = 10;

static NETWORK_GRID_X: u64 = 8;
static EXPECTED_SERVER_HEIGHT: u64 = 10;

pub struct Drawio {
    entries: Vec<String>,
}

impl Drawio {
    fn create_geometry(geometry: &[u64; 4]) -> String {
        format!(
            "<mxGeometry x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" as=\"geometry\"/>",
            geometry[0], geometry[1], geometry[2], geometry[3]
        )
    }

    fn mx_group_params(&mut self, id: String, geometry: &[u64; 4], parent: &String) {
        self.entries.push(format!("<mxCell id=\"{}\" value=\"\" style=\"group;border=2px;\" parent=\"{}\" vertex=\"1\" connectable=\"0\">{}</mxCell>",
        id, parent, Drawio::create_geometry(geometry)))
    }

    fn mx_cell_params(&mut self, id: String, geometry: &[u64; 4], parent: &String, value: String) {
        self.entries.push(format!("<mxCell id=\"{}\" value=\"{}\" style=\"whiteSpace=wrap;html=1;aspect=fixed;fontSize=12;\" parent=\"{}\" vertex=\"1\">{}</mxCell>",
        id, value, parent, Drawio::create_geometry(geometry)));
    }

    pub fn server(
        &mut self,
        server: server::Server,
        location: &[u64; 2],
        parent: &String,
        id: String,
    ) {
        // create group
        let geometry = [
            location[0],
            location[1],
            SERVER_ENTRY_WIDTH,
            SERVER_ENTRY_HEIGHT * server.items.len() as u64,
        ];
        self.mx_group_params(format!("{}-0", id), &geometry, parent);

        // add elements to group
        let mut i = 1;
        let mut item_geometry = [
            0,
            SERVER_ENTRY_HEIGHT * (i - 1),
            SERVER_ENTRY_WIDTH,
            SERVER_ENTRY_HEIGHT,
        ];
        for item in server.items {
            item_geometry[1] = SERVER_ENTRY_HEIGHT * (i - 1);

            self.mx_cell_params(
                format!("{}-{}", id, i),
                &item_geometry,
                &format!("{}-0", id),
                item.value(),
            );
            i += 1;
        }
    }

    pub fn network(
        &mut self,
        servers: Vec<server::Server>,
        location: &[u64; 2],
        parent: &String,
        id: String,
    ) -> u64 {
        let mut index = 0;

        let network_geometry = [
            location[0],
            location[1],
            NETWORK_GRID_X * (SERVER_ENTRY_WIDTH + SERVER_PADDING) + SERVER_PADDING,
            ((servers.len() as u64 / NETWORK_GRID_X) + 1)
                * EXPECTED_SERVER_HEIGHT
                * SERVER_ENTRY_HEIGHT,
        ];

        self.mx_cell_params(
            format!("network-{}-bound", id),
            &network_geometry,
            parent,
            "".to_string(),
        );

        for server in servers.clone() {
            let server_location = [
                location[0]
                    + SERVER_PADDING
                    + (index % NETWORK_GRID_X) * (SERVER_ENTRY_WIDTH + SERVER_PADDING),
                location[1]
                    + SERVER_PADDING
                    + (index / NETWORK_GRID_X) * EXPECTED_SERVER_HEIGHT * SERVER_ENTRY_HEIGHT,
            ];

            self.server(
                server,
                &server_location,
                parent,
                format!("network-{}-{}", id, index),
            );
            index += 1;
        }
        location[1]
            + (SERVER_PADDING * 2)
            + ((servers.len() as u64 / NETWORK_GRID_X) + 1)
                * EXPECTED_SERVER_HEIGHT
                * SERVER_ENTRY_HEIGHT
    }

    pub fn service(
        &mut self,
        service: detail::Service,
        location: &[u64; 2],
        parent: &String,
        id: String,
    ) {
        // create group
        let geometry = [
            location[0],
            location[1],
            IP_ENTRY_WIDTH + PORT_ENTRY_WIDTH,
            SERVER_ENTRY_HEIGHT * (service.hosts.len() + 3) as u64,
        ];
        self.mx_group_params(format!("{}-0", id), &geometry, parent);

        let header_geometry = [
            0,
            0,
            IP_ENTRY_WIDTH + PORT_ENTRY_WIDTH,
            SERVER_ENTRY_HEIGHT * 3,
        ];
        self.mx_cell_params(
            format!("header-{}-0", id),
            &header_geometry,
            &format!("{}-0", id),
            format!(
                "{}\n({} {})",
                service.service,
                service.product,
                service.version.unwrap_or("unknown".to_string())
            ),
        );

        // add elements to group
        let mut i = 1;
        let mut ip_geometry = [0, 0, IP_ENTRY_WIDTH, SERVER_ENTRY_HEIGHT];
        let mut port_geometry = [IP_ENTRY_WIDTH, 0, PORT_ENTRY_WIDTH, SERVER_ENTRY_HEIGHT];
        for item in service.hosts {
            ip_geometry[1] = SERVER_ENTRY_HEIGHT * 3 + SERVER_ENTRY_HEIGHT * (i - 1);
            port_geometry[1] = SERVER_ENTRY_HEIGHT * 3 + SERVER_ENTRY_HEIGHT * (i - 1);

            self.mx_cell_params(
                format!("{}-{}a", id, i),
                &ip_geometry,
                &format!("{}-0", id),
                item.ip,
            );
            self.mx_cell_params(
                format!("{}-{}b", id, i),
                &port_geometry,
                &format!("{}-0", id),
                format!("{}", item.port),
            );
            i += 1;
        }
    }

    pub fn new() -> Drawio {
        let mut instance = Drawio {
            entries: Vec::new(),
        };

        instance.entries.push("<mxGraphModel dx=\"3924\" dy=\"2527\" grid=\"1\" gridSize=\"10\" guides=\"1\" tooltips=\"1\" connect=\"1\" arrows=\"1\" fold=\"1\" page=\"1\" pageScale=\"1\" pageWidth=\"1169\" pageHeight=\"827\" math=\"0\" shadow=\"0\"><root><mxCell id=\"0\"/><mxCell id=\"1\" parent=\"0\"/>".to_string());

        instance
    }

    pub fn export(&mut self, filename: String) {
        self.entries.push("</root></mxGraphModel>".to_string());

        let mut writer = File::create(filename).expect("Could not create file!");
        write!(writer, "{}", self.entries.join("")).expect("Could not write to file!");
    }
}
