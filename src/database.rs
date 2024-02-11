use std::collections::HashMap;
use std::process::exit;

use chrono::{DateTime, Local};
use log::{debug, info};
use mysql::{params, Pool, PooledConn};
use mysql::prelude::Queryable;

use crate::string_builder;

pub(crate) fn upload_data_egress(pool: Pool, data: HashMap<String, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let stamp_time: DateTime<Local> = Local::now();
    let stamp: String = format!("{}", stamp_time.format("%Y-%m-%d %H:%M:%S"));
    let day: String = format!("{}", stamp_time.format("%Y-%m-%d"));
    let mut conn: PooledConn = pool.get_conn()?;
    debug!("Es wurde erfolgreich eine Connection zur Datenbank hergestellt");
    info!("Es werden folgende Trafficdaten als Egress in die Datenbank geschrieben:{:?}",data);
    for (ipaddr, bytes) in data.iter() {
        conn.exec_drop(
            "INSERT INTO RawTraffic (ip, egress, day, updated) VALUES (:ip, :egress, :day, :updated) ON DUPLICATE KEY UPDATE egress = egress + :egress, updated = :updated",
            params! {
            "ip" => ipaddr.to_string().clone(),
            "egress" => bytes,
                "day" => day.clone(),
                "updated" => stamp.clone(),
        },
        )?;
    }
    Ok(())
}

pub(crate) fn upload_data_ingress(pool: Pool, data: HashMap<String, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let stamp_time: DateTime<Local> = Local::now();
    let stamp: String = format!("{}", stamp_time.format("%Y-%m-%d %H:%M:%S"));
    let day: String = format!("{}", stamp_time.format("%Y-%m-%d"));
    let mut conn: PooledConn = pool.get_conn()?;
    debug!("Es wurde erfolgreich eine Connection zur Datenbank hergestellt");
    info!("Es werden folgende Trafficdaten als Ingress in die Datenbank geschrieben:{:?}",data);
    for (ipaddr, bytes) in data.iter() {
        conn.exec_drop(
            "INSERT INTO RawTraffic (ip, ingress, day, updated) VALUES (:ip, :ingress, :day, :updated) ON DUPLICATE KEY UPDATE ingress = ingress + :ingress, updated = :updated",
            params! {
            "ip" => ipaddr.to_string().clone(),
            "ingress" => bytes,
                "day" => day.clone(),
                "updated" => stamp.clone(),
        },
        )?;
    }
    Ok(())
}