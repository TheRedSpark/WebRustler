use std::env;
use std::thread::sleep;
use std::time;

use env_logger::{Builder, WriteStyle};
use log::{ info, LevelFilter};
use crate::dpi::dpi_main;


mod variables;
mod legacy;
mod dpi;
mod database;

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
    //legacy_main();
    dpi_main();
}


pub(crate) fn string_builder() -> String {
    let mysql_ipaddr: String = variables::mysql_ip();
    let mysql_user: String = variables::mysql_user();
    let mysql_database: String = variables::mysql_database();
    let mysql_passwort: String = variables::mysql_passwort();
    let url: String = format!("mysql://{mysql_user}:{mysql_passwort}@{mysql_ipaddr}:3306/{mysql_database}");
    return url.to_string();
}


