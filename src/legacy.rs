use std::collections::HashMap;
use std::process::exit;
use chrono::{DateTime, Local};
use default_net::get_default_interface;
use log::{debug, info, trace, warn};
use mysql::{params, Pool, PooledConn};
use mysql::prelude::Queryable;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use crate::string_builder;

pub(crate) fn legacy_main() {
    let mut data: HashMap<String, usize> = HashMap::new(); //Hashmap<IP,traffic>
    let interface_name: String = get_interface();
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;
    debug!("Das gewählte Interface ist:{}",interface_name);
    let interfaces: Vec<NetworkInterface> = datalink::interfaces();
    let interface: NetworkInterface = interfaces.into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };
    let mut i: i64 = 0;
    loop {
        match rx.next() {
            Ok(packet) => {
                trace!("Das empfangene Paket ist:{:?}",packet);
                parse_packet(packet, &mut data);
                if (i == 100000) {
                    warn!("Daten werden in DB geschrieben");
                    upload_data(data.clone()).unwrap();
                    data.clear();
                    i = 0
                } else {
                    i = i + 1;
                    if i % 1000 == 0 { info!("{}",i) }
                    trace!("Die Paketdurchlaufschleife ist: {}",i)
                }
            }
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn get_interface() -> String {
    return get_default_interface().unwrap().name;
}

fn parse_packet(packet: &[u8], data: &mut HashMap<String, usize>) {
    if let Some(ethernet_packet) = EthernetPacket::new(packet) {
        debug!(
            "Das Paket kam von Mac: {} und geht nach Mac: {}",
            ethernet_packet.get_source(),
            ethernet_packet.get_destination()
        );
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    handle_ipv4_packet(&ipv4_packet, data);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6_packet) = Ipv6Packet::new(ethernet_packet.payload()) {
                    handle_ipv6_packet(&ipv6_packet, &mut Default::default())
                }
            }
            _ => debug!("Das Ethernet-Paket enthält kein IPv4- oder IPv6-Paket."),
        }
    } else {
        debug!("Fehler beim Parsen des Ethernet-Pakets");
    }
}

fn handle_ipv4_packet(ipv4_packet: &Ipv4Packet, data: &mut HashMap<String, usize>) {
    debug!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());
    traffic_count_legacy(data, ipv4_packet.get_source().to_string(), ipv4_packet.get_total_length());
    match ipv4_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                debug!("TCP-Paket: Quellport {}, Zielport {}", tcp_packet.get_source(), tcp_packet.get_destination());
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                debug!("UDP-Paket: Quellport {}, Zielport {}", udp_packet.get_source(), udp_packet.get_destination());
            }
        }
        IpNextHeaderProtocols::Tlsp => { todo!("Protokoll Tlsp muss noch implementiert werden") }
        IpNextHeaderProtocols::Sctp => { todo!("Protokoll Sctp muss noch implementiert werden") }
        _ => debug!("Anderes Protokoll"),
    }
}

fn handle_ipv6_packet(ipv6_packet: &Ipv6Packet, data: &mut HashMap<String, usize>) {
    debug!("IPv6: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv6_packet.get_source(),ipv6_packet.get_destination(),ipv6_packet.get_payload_length());
    traffic_count_legacy(data, ipv6_packet.get_source().to_string(), ipv6_packet.get_payload_length())
    //todo!("IPv6 Protokolle implementieren")
}

fn traffic_count_legacy(data: &mut HashMap<String, usize>, ip_addr: String, traffic_packet: u16) {
    *data.entry(ip_addr).or_insert(0) += traffic_packet as usize;
    debug!("Die Trafficdaten sind:{:?}",data);
}

fn upload_data(data: HashMap<String, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let db_pool: Pool = Pool::new(&*string_builder()).expect("Pool bildung fehlgeschlagen");
    let stamp: DateTime<Local> = Local::now();
    let stamp: String = format!("{}", stamp.format("%Y-%m-%d %H:%M:%S"));
    let mut conn: PooledConn = db_pool.get_conn()?;
    debug!("Es wurde erfolgreich eine Connection zur Datenbank hergestellt");
    info!("Es werden folgende Trafficdaten in die Datenbank geschrieben:{:?}",data);
    for (ipaddr, bytes) in data.iter() {
        conn.exec_drop(
            "INSERT INTO Test (ip_src, bytes,stamp_inserted,stamp_updated) VALUES (:ip, :bytes, :stamp, :stamp) ON DUPLICATE KEY UPDATE bytes = bytes + :bytes,stamp_updated = :stamp",
            params! {
            "ip" => ipaddr.to_string().clone(),
            "bytes" => bytes,
                "stamp" => stamp.clone(),
        },
        )?;
    }
    exit(0);
    Ok(())
}