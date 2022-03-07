use std::collections::BTreeMap;

mod detail;
mod parser;
mod renderer;
mod server;

/* Line:
<mxCell id="QOGmrZeb03ETIo_Szw0Q-2" style="edgeStyle=orthogonalEdgeStyle;shape=link;rounded=0;orthogonalLoop=1;jettySize=auto;html=1;exitX=0.5;exitY=1;exitDx=0;exitDy=0;entryX=0.5;entryY=1;entryDx=0;entryDy=0;" edge="1" parent="1" source="network-network-1-17-3" target="network-network-1-18-3">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
*/

fn main() {
    let hosts = parser::Parser::parse("./testfiles/output1.xml".to_string());

    // sort server into network categories

    let mut topology: BTreeMap<u64, Vec<&parser::Host>> = BTreeMap::new();
    for host in &hosts {
        topology
            .entry(host.hops.len() as u64)
            .or_insert(Vec::new())
            .push(host);
    }

    let mut canvas = renderer::Drawio::new();
    let mut tables = detail::Tables::new();

    let mut id: u64 = 1;
    let mut used_height = 10;

    for (distance, servers) in topology {
        let itemized_servers = servers
            .iter()
            .map(|s| server::Server::into_items(s.to_owned().to_owned()))
            .collect();

        used_height = canvas.network(
            itemized_servers,
            &[10, used_height],
            &"1".to_string(),
            format!("network-{}", distance),
        ) + 10;
        id += 1;
    }

    for host in &hosts {
        tables.add_host(&host);
    }

    //println!("{:#?}", tables.services);
    for service in tables.services {
        let location = [
            1000 + id * (renderer::IP_ENTRY_WIDTH + renderer::PORT_ENTRY_WIDTH + 30),
            renderer::SERVER_ENTRY_HEIGHT,
        ];

        canvas.service(service, &location, &"1".to_string(), format!("table{}", id));
        id += 1;
    }

    canvas.export("./export.drawio".to_string());
}
