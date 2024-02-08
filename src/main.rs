use std::env;
use std::thread::sleep;
use std::time;

use default_net::get_default_interface;
use env_logger::{Builder, WriteStyle};
use log::{debug, info, LevelFilter, trace};
use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_NAME");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

fn welcome() {
    let speed: u64 = 40;
    println!(r"          _____                    _____          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"         /\    \                  /\    \         ");
    sleep(time::Duration::from_millis(speed));
    println!(r"        /::\____\                /::\    \        ");
    sleep(time::Duration::from_millis(speed));
    println!(r"       /:::/    /               /::::\    \       ");
    sleep(time::Duration::from_millis(speed));
    println!(r"      /:::/   _/___            /::::::\    \      ");
    sleep(time::Duration::from_millis(speed));
    println!(r"     /:::/   /\    \          /:::/\:::\    \     ");
    sleep(time::Duration::from_millis(speed));
    println!(r"    /:::/   /::\____\        /:::/__\:::\    \    ");
    sleep(time::Duration::from_millis(speed));
    println!(r"   /:::/   /:::/    /       /::::\   \:::\    \   ");
    sleep(time::Duration::from_millis(speed));
    println!(r"  /:::/   /:::/   _/___    /::::::\   \:::\    \  ");
    sleep(time::Duration::from_millis(speed));
    println!(r" /:::/___/:::/   /\    \  /:::/\:::\   \:::\____\ ");
    sleep(time::Duration::from_millis(speed));
    println!(r"|:::|   /:::/   /::\____\/:::/  \:::\   \:::|    |");
    sleep(time::Duration::from_millis(speed));
    println!(r"|:::|__/:::/   /:::/    /\::/   |::::\  /:::|____|");
    sleep(time::Duration::from_millis(speed));
    println!(r" \:::\/:::/   /:::/    /  \/____|:::::\/:::/    / ");
    sleep(time::Duration::from_millis(speed));
    println!(r"  \::::::/   /:::/    /         |:::::::::/    /  ");
    sleep(time::Duration::from_millis(speed));
    println!(r"   \::::/___/:::/    /          |::|\::::/    /   ");
    sleep(time::Duration::from_millis(speed));
    println!(r"    \:::\__/:::/    /           |::| \::/____/    ");
    sleep(time::Duration::from_millis(speed));
    println!(r"     \::::::::/    /            |::|  ~|          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"      \::::::/    /             |::|   |          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"       \::::/    /              \::|   |          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"        \::/____/                \:|   |          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"         ~~                       \|___|          ");
    sleep(time::Duration::from_millis(speed));
    println!(r"                                                  ");
    sleep(time::Duration::from_millis(speed));
}

fn setup() {
    welcome();
    let mut builder: Builder = Builder::new();
    builder
        .filter(None, LevelFilter::Debug)
        .write_style(WriteStyle::Always)
        .init();
    info!("The software is licensed under {}. All rights reserved",LICENSE);
    info!("{} started with version {}",NAME,VERSION);
}

fn main() {
    setup();
    test();
}


fn test() {
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

    loop {
        match rx.next() {
            Ok(packet) => {
                trace!("Das empfangene Paket ist:{:?}",packet);
                parse_packet(packet);
            }
            Err(e) => {
                // If an error occurs, we can handle it here
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn get_interface() -> String {
    return get_default_interface().unwrap().name;
}

fn parse_packet(packet: &[u8]) {
    if let Some(ethernet_packet) = EthernetPacket::new(packet) {
        debug!(
            "Das Paket kam von Mac: {} und geht nach Mac: {}",
            ethernet_packet.get_source(),
            ethernet_packet.get_destination()
        );
        match ethernet_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4_packet) = Ipv4Packet::new(ethernet_packet.payload()) {
                    handle_ipv4_packet(&ipv4_packet);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6_packet) = Ipv6Packet::new(ethernet_packet.payload()) {
                    handle_ipv6_packet(&ipv6_packet)
                }
            }
            _ => debug!("Das Ethernet-Paket enthält kein IPv4- oder IPv6-Paket."),
        }
    } else {
        debug!("Fehler beim Parsen des Ethernet-Pakets");
    }
}

fn handle_ipv4_packet(ipv4_packet: &Ipv4Packet) {
    debug!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());

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
        _ => debug!("Anderes Protokoll"),
    }
}

fn handle_ipv6_packet(ipv6_packet: &Ipv6Packet) {
    debug!("IPv6: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv6_packet.get_source(),ipv6_packet.get_destination(),ipv6_packet.get_payload_length());

    todo!()
}
