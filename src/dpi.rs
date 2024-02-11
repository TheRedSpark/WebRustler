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

use crate::database::{ upload_data_ingress, upload_data_with_key};
use crate::string_builder;

const SUBNET: u32 = u32::from_be_bytes(Ipv4Addr::new(10, 82, 62, 0).octets());
const MASK: u32 = u32::from_be_bytes(Ipv4Addr::new(255, 255, 255, 0).octets());

pub(crate) fn dpi_main() {
    let mut data: HashMap<String, &mut HashMap<String, usize>> = Default::default();
    let mut ingress: HashMap<String, usize> = Default::default();
    let mut egress: HashMap<String, usize> = Default::default();
    let mut tlsp: HashMap<String, usize> = Default::default();
    let mut sctp: HashMap<String, usize> = Default::default();
    let mut tcp: HashMap<String, usize> = Default::default();
    let mut udp: HashMap<String, usize> = Default::default();
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
                parse_packet(packet, &mut data);
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
        trace!(
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
    trace!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());
    //traffic_count(data, &ipv4_packet.get_source(), &ipv4_packet.get_destination(), ipv4_packet.get_total_length());
    match ipv4_packet.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                trace!("TCP-Paket: Quellport {}, Zielport {}", tcp_packet.get_source(), tcp_packet.get_destination());
                traffic_count(data, &ipv4_packet.get_source(), &ipv4_packet.get_destination(), ipv4_packet.get_total_length(),"tcp");
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp_packet) = UdpPacket::new(ipv4_packet.payload()) {
                trace!("UDP-Paket: Quellport {}, Zielport {}", udp_packet.get_source(), udp_packet.get_destination());
                traffic_count(data, &ipv4_packet.get_source(), &ipv4_packet.get_destination(), ipv4_packet.get_total_length(),"udp");
            }
        }
        IpNextHeaderProtocols::Tlsp => { todo!("Protokoll Tlsp muss noch implementiert werden") }
        IpNextHeaderProtocols::Sctp => { todo!("Protokoll Sctp muss noch implementiert werden") }
        _ => debug!("Anderes Protokoll"),
    }
}

fn handle_ipv6_packet(ipv6_packet: &Ipv6Packet, _data: &mut HashMap<String, usize>) {
    debug!("IPv6: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv6_packet.get_source(),ipv6_packet.get_destination(),ipv6_packet.get_payload_length());
    //todo!("IPv6 Protokolle implementieren")
}

fn upload_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    let pool: Pool = Pool::new(&*string_builder()).expect("Pool bildung fehlgeschlagen");
    for (key, value) in data.iter_mut() {
        //let key: String = key.to_string();
        upload_data_with_key(pool.clone(), value.clone(),key).unwrap()
        // match key.as_str() {
        //     "egress" => { upload_data_egress(pool.clone(), value.clone()).unwrap() }
        //     "ingress" => { upload_data_ingress(pool.clone(), value.clone()).unwrap() }
        //     &_ => { panic!() }
        // }
    }
}

fn clear_data(data: &mut HashMap<String, &mut HashMap<String, usize>>) {
    for (key, value) in data.iter_mut() {
        println!("Schlüssel: {}, Wert: {:?}", key, value);
        value.clear();
    }
}

fn traffic_count(data: &mut HashMap<String, &mut HashMap<String, usize>>, ip_source: &Ipv4Addr, ip_destination: &Ipv4Addr, traffic_packet: u16, key_protokoll: &str) {
    let ip = Ipv4Addr::from_str(&*ip_source.to_string()).unwrap();
    if ip_belongs_to_subnet(ip) {
        if let Some(traffic_raw) = data.get_mut("egress") {
            *traffic_raw.entry((*ip_source).to_string()).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
        if let Some(traffic_raw) = data.get_mut(key_protokoll) {
            *traffic_raw.entry((*ip_source).to_string()).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
    } else {
        if let Some(traffic_raw) = data.get_mut("ingress") {
            *traffic_raw.entry((*ip_destination).to_string()).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
        if let Some(traffic_raw) = data.get_mut(key_protokoll) {
            *traffic_raw.entry((*ip_destination).to_string()).or_insert(0) += traffic_packet as usize;
        } else {
            panic!()
        }
    }


    trace!("Die Trafficdaten sind:{:?}", data);
}

fn ip_belongs_to_subnet(ip: Ipv4Addr) -> bool {
    let ip_int = u32::from_be_bytes(ip.octets());

    (ip_int & MASK) == (SUBNET & MASK)
}

