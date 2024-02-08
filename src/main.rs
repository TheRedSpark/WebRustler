use std::thread::sleep;
use std::time;

use env_logger::{Builder, WriteStyle};
use log::{info, LevelFilter};

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

fn main() {
    welcome();
    let mut builder = Builder::new();
    builder
        .filter(None, LevelFilter::Info)
        .write_style(WriteStyle::Always)
        .init();
    info!("The software is licensed under {}. All rights reserved",LICENSE)
    info!("{} started with version {}",NAME,VERSION);
}
