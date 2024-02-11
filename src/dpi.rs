//Trafic soll allgemein gespeichert werden nach Menge und dann noch nach:
//Dienst(Streaming Media,Network Protokolls,Web,Games,File Transfer,Social Network,P2P,VoIP,Mail,VPN,)
//Genaue dienst Aufspaltung(Insta,YT,etc.)

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::ops::DerefMut;
use std::process::exit;
use std::str::FromStr;

use default_net::get_default_interface;
use env_logger::fmt::style::AnsiColor::Magenta;
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

use crate::database::{upload_data_egress, upload_data_ingress};
use crate::string_builder;
const SUBNET:u32 = u32::from_be_bytes(Ipv4Addr::new(10,82,62,0).octets());
const MASK:u32 = u32::from_be_bytes(Ipv4Addr::new(255, 255, 255, 0).octets());

pub(crate) fn dpi_main() {
    let mut data: HashMap<String, &mut HashMap<String, usize>> = Default::default();
    let mut ingress: HashMap<String, usize> = Default::default();
    ingress.insert("127.0.0.1".to_string(), 30);
    let mut egress: HashMap<String, usize> = Default::default();
    data.insert("ingress".to_string(), &mut ingress);
    data.insert("egress".to_string(), &mut egress);
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
                parse_packet(packet, &mut data);
                if (i == 5000) {
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
    let interface_name: String = get_interface();
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

fn get_interface() -> String {
    return get_default_interface().unwrap().name;
}

fn parse_packet(packet: &[u8], data: &mut HashMap<String, &mut HashMap<String, usize>>) {
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

fn handle_ipv4_packet(ipv4_packet: &Ipv4Packet, data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    debug!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());
    traffic_count(data, ipv4_packet.get_source().to_string(), ipv4_packet.get_destination().to_string(), ipv4_packet.get_total_length());
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
    //todo!("IPv6 Protokolle implementieren")
}

fn upload_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    let pool: Pool = Pool::new(&*string_builder()).expect("Pool bildung fehlgeschlagen");
    for (key, value) in data.iter_mut() {
        let key: String = key.to_string();
        match key.as_str() {
            "egress" => { upload_data_egress(pool.clone(), value.clone()).unwrap() }
            "ingress" => { upload_data_ingress(pool.clone(), value.clone()).unwrap() }
            &_ => { panic!() }
        }
    }
    exit(0)
}

fn clear_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    for (key, value) in data.iter_mut() {
        println!("Schlüssel: {}, Wert: {:?}", key, value);
        value.clear();
    }
}

fn traffic_count(data: &mut HashMap<String, &mut HashMap<String, usize>>, ip_source: String, ip_destination: String, traffic_packet: u16) {
    let mut key: &str;
    let ip = Ipv4Addr::from_str(&*ip_source).unwrap();
    if ip_belongs_to_subnet(ip, &subnet, &mask) { key = "egress";
        if let Some(traffic_raw) = data.get_mut(key) {
            *traffic_raw.entry(ip_source).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
    } else { key = "ingress";
        if let Some(traffic_raw) = data.get_mut(key) {
            *traffic_raw.entry(ip_destination).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
    }

    debug!("Die Trafficdaten sind:{:?}", data);
}

fn ip_belongs_to_subnet(ip: Ipv4Addr, subnet: Ipv4Addr, mask: Ipv4Addr) -> bool {
    let ip_int = u32::from_be_bytes(ip.octets());

    // Berechnung der Netzwerkadresse des Subnetzes und der IP
    (ip_int & MASK) == (SUBNET & MASK)
}

