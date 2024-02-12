//Trafic soll allgemein gespeichert werden nach Menge und dann noch nach:
//Dienst(Streaming Media,Network Protokolls,Web,Games,File Transfer,Social Network,P2P,VoIP,Mail,VPN,)
//Genaue dienst Aufspaltung(Insta,YT,etc.)

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::exit;
use std::str::FromStr;

use default_net::get_default_interface;
use log::{debug, info, trace, warn};
use mysql::Pool;
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

use crate::database::{upload_data_ingress, upload_data_with_key};
use crate::string_builder;

const SUBNET: u32 = u32::from_be_bytes(Ipv4Addr::new(10, 82, 62, 0).octets());
const MASK: u32 = u32::from_be_bytes(Ipv4Addr::new(255, 255, 255, 0).octets());
const IPV4DEFAULT: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

struct PacketPropertys {
    is_egress: bool,
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    lokal_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    lokal_port: u16,
    length: u16,
    ether_type: String,
    ip_next_header_protocol: String,
    iana_protocol: String,
    ttl: u8,
    version: u8,

}

pub(crate) fn dpi_main() {
    let mut data: HashMap<String, &mut HashMap<String, usize>> = Default::default();
    let mut ingress: HashMap<String, usize> = Default::default();
    let mut egress: HashMap<String, usize> = Default::default();
    let mut tlsp: HashMap<String, usize> = Default::default();
    let mut sctp: HashMap<String, usize> = Default::default();
    let mut tcp: HashMap<String, usize> = Default::default();
    let mut udp: HashMap<String, usize> = Default::default();
    let mut ftp: HashMap<String, usize> = Default::default();
    let mut ssh: HashMap<String, usize> = Default::default();
    let mut telnet: HashMap<String, usize> = Default::default();
    let mut smtp: HashMap<String, usize> = Default::default();
    let mut dns: HashMap<String, usize> = Default::default();
    let mut http: HashMap<String, usize> = Default::default();
    let mut pop3: HashMap<String, usize> = Default::default();
    let mut imap: HashMap<String, usize> = Default::default();
    let mut https: HashMap<String, usize> = Default::default();
    let mut mysql: HashMap<String, usize> = Default::default();
    let mut rdp: HashMap<String, usize> = Default::default();
    let mut vnc: HashMap<String, usize> = Default::default();
    let mut unknown: HashMap<String, usize> = Default::default();
    data.insert("ftp".to_string(), &mut ftp);
    data.insert("ssh".to_string(), &mut ssh);
    data.insert("telnet".to_string(), &mut telnet);
    data.insert("smtp".to_string(), &mut smtp);
    data.insert("dns".to_string(), &mut dns);
    data.insert("http".to_string(), &mut http);
    data.insert("pop3".to_string(), &mut pop3);
    data.insert("imap".to_string(), &mut imap);
    data.insert("https".to_string(), &mut https);
    data.insert("mysql".to_string(), &mut mysql);
    data.insert("rdp".to_string(), &mut rdp);
    data.insert("vnc".to_string(), &mut vnc);
    data.insert("unknown".to_string(), &mut unknown);
    data.insert("ingress".to_string(), &mut ingress);
    data.insert("egress".to_string(), &mut egress);
    data.insert("tlsp".to_string(), &mut tlsp);
    data.insert("sctp".to_string(), &mut sctp);
    data.insert("tcp".to_string(), &mut tcp);
    data.insert("udp".to_string(), &mut udp);
    let interface = get_used_interface();
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
                let mut packetpropertys: PacketPropertys = PacketPropertys {
                    is_egress: false,
                    src_ip: IPV4DEFAULT,
                    dst_ip: IPV4DEFAULT,
                    lokal_ip: IPV4DEFAULT,
                    src_port: 0,
                    dst_port: 0,
                    lokal_port: 0,
                    length: 0,
                    ether_type: "".to_string(),
                    ip_next_header_protocol: "".to_string(),
                    iana_protocol: "".to_string(),
                    ttl: 0,
                    version: 0,
                };
                parse_packet(packet, &mut packetpropertys);
                analyze_packet(&mut packetpropertys);
                count_data(&mut packetpropertys, &mut data);


                if i == 5000 {
                    warn!("Daten werden in DB geschrieben");
                    upload_data(&mut data);
                    clear_data(&mut data);
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


fn get_used_interface() -> NetworkInterface {
    let interface_name: String = get_default_interface().unwrap().name;
    let interface_names_match =
        |iface: &NetworkInterface| iface.name == interface_name;
    debug!("Das gewählte Interface ist:{}",interface_name);
    let interfaces: Vec<NetworkInterface> = datalink::interfaces();
    let interface: NetworkInterface = interfaces.into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();
    return interface;
}

fn parse_packet(packet: &[u8], packetpropertys: &mut PacketPropertys) {
    if let Some(ethernet_packet) = EthernetPacket::new(packet) {
        trace!(
            "Das Paket kam von Mac: {} und geht nach Mac: {}",
            ethernet_packet.get_source(),
            ethernet_packet.get_destination()
        );
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    packetpropertys.ether_type = "Ipv4".to_string();
                    handle_ipv4_packet(&ipv4_packet, packetpropertys);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6_packet) = Ipv6Packet::new(ethernet_packet.payload()) {
                    handle_ipv6_packet(&ipv6_packet, &mut Default::default())
                }
            }
            EtherTypes::Arp => {}
            EtherTypes::Ipx => {}
            EtherTypes::Arp => {}
            EtherTypes::Lldp => {}
            _ => debug!("Das Ethernet-Paket enthält kein IPv4- oder IPv6-Paket. sondern ein {}",ethernet_packet.get_ethertype()),
        }
    } else {
        debug!("Fehler beim Parsen des Ethernet-Pakets");
    }
}

fn handle_ipv4_packet(ipv4_packet: &Ipv4Packet, packetpropertys: &mut PacketPropertys) {
    trace!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());
    packetpropertys.version = ipv4_packet.get_version();
    packetpropertys.ttl = ipv4_packet.get_ttl();
    packetpropertys.length = ipv4_packet.get_total_length();
    packetpropertys.ip_next_header_protocol = ipv4_packet.get_next_level_protocol().to_string();
    packetpropertys.dst_ip = ipv4_packet.get_destination();
    packetpropertys.src_ip = ipv4_packet.get_source();
    match ipv4_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                trace!("TCP-Paket: Quellport {}, Zielport {}", tcp_packet.get_source(), tcp_packet.get_destination());
                packetpropertys.src_port = tcp_packet.get_source();
                packetpropertys.dst_port = tcp_packet.get_destination();
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                trace!("UDP-Paket: Quellport {}, Zielport {}", udp_packet.get_source(), udp_packet.get_destination());
                packetpropertys.src_port = udp_packet.get_source();
                packetpropertys.dst_port = udp_packet.get_destination();
            }
        }
        IpNextHeaderProtocols::Tlsp => { todo!("Protokoll Tlsp muss noch implementiert werden") }
        IpNextHeaderProtocols::Sctp => { todo!("Protokoll Sctp muss noch implementiert werden") }
        _ => debug!("Anderes Protokoll"),
    }
}

fn handle_ipv6_packet(ipv6_packet: &Ipv6Packet, _data: &mut HashMap<String, usize>) {
    trace!("IPv6: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv6_packet.get_source(),ipv6_packet.get_destination(),ipv6_packet.get_payload_length());
    //todo!("IPv6 Protokolle implementieren")
}

fn upload_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    let pool: Pool = Pool::new(&*string_builder()).expect("Pool bildung fehlgeschlagen");
    for (key, value) in data.iter_mut() {
        upload_data_with_key(pool.clone(), value.clone(), key).unwrap()
    }
}

fn clear_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    for (key, value) in data.iter_mut() {
        debug!("Schlüssel: {}, Wert: {:?}", key, value);
        value.clear();
    }
}

fn traffic_count(data: &mut HashMap<String, &mut HashMap<String, usize>>, key_protokoll: &str, packet_propertys: &mut PacketPropertys) {
    if let Some(traffic_raw) = data.get_mut(key_protokoll) {
        *traffic_raw.entry((*packet_propertys.lokal_ip.to_string()).to_string()).or_insert(0) += packet_propertys.length as usize;
    } else {
        panic!()
    }
    if let Some(traffic_raw) = data.get_mut(packet_propertys.iana_protocol.as_str()) {
        *traffic_raw.entry((*packet_propertys.lokal_ip.to_string()).to_string()).or_insert(0) += packet_propertys.length as usize;
    } else {
        warn!("{:?}",data.get_mut(packet_propertys.iana_protocol.as_str()))
    }
    trace!("Die Trafficdaten sind:{:?}", data);
}

fn ip_belongs_to_subnet(ip: Ipv4Addr) -> bool {
    let ip_int = u32::from_be_bytes(ip.octets());

    (ip_int & MASK) == (SUBNET & MASK)
}

fn count_data(packet_propertys: &mut PacketPropertys, data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    let key = packet_propertys.iana_protocol.as_str();
    if packet_propertys.is_egress {
        traffic_count(data, "egress", packet_propertys)
    } else { traffic_count(data, "ingress", packet_propertys) }
    //traffic_count(data, key, packet_propertys)
}

fn analyze_packet(packet_propertys: &mut PacketPropertys) {
    if ip_belongs_to_subnet(packet_propertys.src_ip) {
        packet_propertys.lokal_ip = packet_propertys.src_ip;
        packet_propertys.is_egress = true;
        packet_propertys.lokal_port = packet_propertys.dst_port
    } else {
        packet_propertys.lokal_ip = packet_propertys.dst_ip;
        packet_propertys.is_egress = false;
        packet_propertys.lokal_port = packet_propertys.src_port
    };
    match packet_propertys.lokal_port {
        21 => { packet_propertys.iana_protocol = "ftp".to_string() }
        22 => { packet_propertys.iana_protocol = "ssh".to_string() }
        23 => { packet_propertys.iana_protocol = "telnet".to_string() }
        25 => { packet_propertys.iana_protocol = "smtp".to_string() }
        53 => { packet_propertys.iana_protocol = "dns".to_string() }
        80 => { packet_propertys.iana_protocol = "http".to_string() }
        110 => { packet_propertys.iana_protocol = "pop3".to_string() }
        143 => { packet_propertys.iana_protocol = "imap".to_string() }
        443 => { packet_propertys.iana_protocol = "https".to_string() }
        3306 => { packet_propertys.iana_protocol = "mysql".to_string() }
        3389 => { packet_propertys.iana_protocol = "rdp".to_string() }
        5900 => { packet_propertys.iana_protocol = "vnc".to_string() }
        _ => { packet_propertys.iana_protocol = "unknown".to_string() }
    }
}



