use std::collections::HashMap;
use std::env;
use std::net::Ipv4Addr;
use std::process::exit;
use std::thread::sleep;
use std::time;

use chrono::{DateTime, Local};
use default_net::get_default_interface;
use env_logger::{Builder, WriteStyle};
use log::{debug, info, LevelFilter, trace, warn};
use mysql::{params, Pool, PooledConn};
use mysql::prelude::Queryable;
use pnet::datalink::{self, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;

mod variables;

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
        .filter(None, LevelFilter::Info)
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
    let mut data: HashMap<Ipv4Addr, usize> = HashMap::new(); //Hashmap<IP,traffic>
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
                if (i == 1000000) {
                    warn!("Daten werden in DB geschrieben");
                    upload_data(data.clone()).unwrap();
                    data.clear();
                    i = 0
                } else {
                    i = i + 1;
                    if i % 1000 == 0 { info!("{}",i) }
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

fn parse_packet(packet: &[u8], data: &mut HashMap<Ipv4Addr, usize>) {
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
                    handle_ipv6_packet(&ipv6_packet)
                }
            }
            _ => debug!("Das Ethernet-Paket enthält kein IPv4- oder IPv6-Paket."),
        }
    } else {
        debug!("Fehler beim Parsen des Ethernet-Pakets");
    }
}

fn handle_ipv4_packet(ipv4_packet: &Ipv4Packet, data: &mut HashMap<Ipv4Addr, usize>) {
    debug!("IPv4: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv4_packet.get_source(),ipv4_packet.get_destination(),ipv4_packet.get_total_length());
    traffic_count_legacy(data, ipv4_packet.get_source(), ipv4_packet.get_total_length());
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

fn handle_ipv6_packet(ipv6_packet: &Ipv6Packet) {
    debug!("IPv6: Sender: {}, Empfänger: {} die Länge des Paketes ist: {}",ipv6_packet.get_source(),ipv6_packet.get_destination(),ipv6_packet.get_payload_length());

    //todo!()
}

fn traffic_count_legacy(data: &mut HashMap<Ipv4Addr, usize>, ip_addr: Ipv4Addr, traffic_packet: u16) {
    *data.entry(ip_addr).or_insert(0) += traffic_packet as usize;
    debug!("Die Trafficdaten sind:{:?}",data);
    //upload_data(data.clone()).unwrap();
    //data.clear()
}

fn upload_data(data: HashMap<Ipv4Addr, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let db_pool: Pool = Pool::new(&*string_builder()).expect("Pool bildung fehlgeschlagen");
    let stamp: DateTime<Local> = Local::now();
    let stamp: String = format!("{}", stamp.format("%Y-%m-%d %H:%M:%S"));
    let mut conn: PooledConn = db_pool.get_conn()?;
    debug!("Es wurde erfolgreich eine Connection zur Datenbank hergestellt");
    info!("Es werden folgende Trafficdaten in die Datenbank geschrieben:{:?}",data);
    //trace!("Es werden nun folgende Daten in die Datenbank geschrieben -> Menge:{},Sorte:{},Prozente:{},Bemerkung:{}",data.menge,data.sorte,data.prozente,data.bemerkung);
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


pub(crate) fn string_builder() -> String {
    let mysql_ipaddr: String = variables::mysql_ip();
    let mysql_user: String = variables::mysql_user();
    let mysql_database: String = variables::mysql_database();
    let mysql_passwort: String = variables::mysql_passwort();
    let url: String = format!("mysql://{mysql_user}:{mysql_passwort}@{mysql_ipaddr}:3306/{mysql_database}");
    return url.to_string();
}


