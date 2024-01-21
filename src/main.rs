//use bluer::{AdapterEvent, Device};
use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::StreamExt;
use std::time::Duration;
use tokio::time;

const APPLE: u16 = 76;
const DATA_LENGTH: usize = 27;
type Packet = [u8; 27];
const MIN_RSSI: i16 = -60;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = read_bytes().await?;
    if let Some(raw) = bytes {
        let pod = Pod::try_parse(&raw).unwrap();
        println!("POD: {:?}", pod);
        println!("BYTES: {:?}", bytes);
    }

    Ok(())
}

async fn read_bytes() -> Result<Option<Packet>, Box<dyn std::error::Error>> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;

    if adapter_list.is_empty() {
        eprintln!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        adapter.start_scan(ScanFilter::default()).await?;
        time::sleep(Duration::from_secs(2)).await;
        let peripherals = adapter.peripherals().await?;

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?.unwrap_or_default();
            let datas = properties.manufacturer_data.get(&APPLE);

            if let Some(data) = datas {
                if data.len() == DATA_LENGTH {
                    let d: Packet = data.clone().try_into().unwrap();
                    return Ok(Some(d));
                }
            }
        }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub enum Model {
    AirPods1,
    AirPods2,
    AirPods3,
    AirPodsPro,
    AirPodsPro2,
    AirPodsMax,
    PowerbeatsPro,
    BeatsX,
    BeatsFlex,
    BeatsSolo3,
    BeatsStudio3,
    Powerbeats3,
    Unknown,
}

impl Model {
    fn parse(hex: &str) -> Model {
        // Extract the single character and the substring from `status`
        let id_single = hex.chars().nth(7).unwrap(); // Safe as we expect `status` to be long enough
        let id_full = &hex[6..10]; // Slices in Rust are zero-indexed and exclusive at the end

        // Pattern match on `id_full` and `id_single` to identify the pod type
        match id_full {
            "0220" => Model::AirPods1,
            "0F20" => Model::AirPods2,
            "1320" => Model::AirPods3,
            "0E20" => Model::AirPodsPro,
            "1420" => Model::AirPodsPro2,
            _ if id_single == 'A' => Model::AirPodsMax,
            _ if id_single == 'B' => Model::PowerbeatsPro,
            "0520" => Model::BeatsX,
            "1020" => Model::BeatsFlex,
            "0620" => Model::BeatsSolo3,
            _ if id_single == '9' => Model::BeatsStudio3,
            "0320" => Model::Powerbeats3,
            _ => Model::Unknown,
        }
    }
}

fn to_hex_str(data: &Packet) -> String {
    data.iter().map(|byte| format!("{:02X}", byte)).collect()
}

#[derive(Debug, Clone)]
pub struct Pod {
    pub model: Model,
    pub left_status: i32,
    pub right_status: i32,
    pub case_status: i32,
    pub single_status: i32,
    pub charge_status: i32,
    pub charge_left: bool,
    pub charge_right: bool,
    pub charge_case: bool,
    pub charge_single: bool,
    pub in_ear_left: bool,
    pub in_ear_right: bool,
}

impl Pod {
    fn try_parse(raw: &Packet) -> Option<Pod> {
        let hex = to_hex_str(raw);
        println!("HEX: {}", hex);

        let flip = false;

        // Parsing battery status with hexadecimal base
        let left_status = i32::from_str_radix(
            &hex.chars()
                .nth(flip as usize * 12 + (!flip as usize) * 13)
                .unwrap()
                .to_string(),
            16,
        )
        .unwrap();

        let right_status = i32::from_str_radix(
            &hex.chars()
                .nth(flip as usize * 13 + (!flip as usize) * 12)
                .unwrap()
                .to_string(),
            16,
        )
        .unwrap();

        let case_status =
            i32::from_str_radix(&hex.chars().nth(15).unwrap().to_string(), 16).unwrap();
        let single_status =
            i32::from_str_radix(&hex.chars().nth(13).unwrap().to_string(), 16).unwrap();

        // Parsing charge status
        let charge_status =
            i32::from_str_radix(&hex.chars().nth(14).unwrap().to_string(), 16).unwrap();

        // Determining charging status with bitwise operations
        let charge_left = (charge_status & if flip { 0b00000010 } else { 0b00000001 }) != 0;
        let charge_right = (charge_status & if flip { 0b00000001 } else { 0b00000010 }) != 0;
        let charge_case = (charge_status & 0b00000100) != 0;
        let charge_single = (charge_status & 0b00000001) != 0;

        // Parsing in-ear status
        let in_ear_status =
            i32::from_str_radix(&hex.chars().nth(11).unwrap().to_string(), 16).unwrap();

        // Determining in-ear status with bitwise operations
        let in_ear_left = (in_ear_status & if flip { 0b00001000 } else { 0b00000010 }) != 0;
        let in_ear_right = (in_ear_status & if flip { 0b00000010 } else { 0b00001000 }) != 0;

        Some(Pod {
            model: Model::parse(&hex),

            left_status,
            right_status,
            case_status,
            single_status,
            charge_status,
            charge_left,
            charge_right,
            charge_case,
            charge_single,
            in_ear_left,
            in_ear_right,
        })
    }
}
