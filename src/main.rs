use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use btleplug::Error;
use std::time::Duration;
use tokio::time;

const APPLE: u16 = 76;
const DATA_LENGTH: usize = 27;
type Packet = [u8; 27];
const MIN_RSSI: i16 = -90;
//const MIN_RSSI: i16 = -60;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pods = find_pods().await?;
    for pod in pods {
        println!("POD: {:#?}", pod);
    }

    Ok(())
}

pub async fn find_pods() -> Result<Vec<Pod>, Error> {
    let bytes = read_bytes().await?;
    Ok(bytes.iter().map(Pod::parse).collect())
}

async fn read_bytes() -> Result<Vec<Packet>, Error> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    let mut found = Vec::default();

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

            if let Some(rssi) = properties.rssi {
                if rssi < MIN_RSSI {
                    continue;
                }
            }

            if let Some(data) = datas {
                if data.len() == DATA_LENGTH {
                    let d: Packet = data.clone().try_into().unwrap();
                    found.push(d);
                }
            }
        }
    }
    Ok(found)
}

#[derive(Debug, Clone)]
pub struct Pod {
    pub model: Model,
    pub left: Device,
    pub right: Device,
    pub case: Option<Device>,
}

impl Pod {
    fn parse(raw: &Packet) -> Pod {
        let hex = to_hex_str(raw);
        let model = Model::parse(&hex);
        let [left, mut right, case] = build_devices(&hex);
        let mut case = Some(case);

        // If it is a single model, the left and right device are the same
        // and there is no case
        let single = match model {
            Model::Unknown => false,
            Model::AirPods1 => false,
            Model::AirPods2 => false,
            Model::AirPods3 => false,
            Model::AirPodsPro => false,
            Model::AirPodsPro2 => false,
            Model::AirPodsPro2Usbc => false,
            Model::AirPodsMax => true,
            Model::PowerbeatsPro => false,
            Model::BeatsX => true,
            Model::BeatsFlex => true,
            Model::BeatsSolo3 => true,
            Model::BeatsStudio3 => true,
            Model::Powerbeats3 => true,
        };

        if single {
            right = left.clone();
            case = None
        }

        Pod {
            model,
            left,
            right,
            case,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Model {
    AirPods1,
    AirPods2,
    AirPods3,
    AirPodsPro,
    AirPodsPro2,
    AirPodsPro2Usbc,
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
            "2420" => Model::AirPodsPro2Usbc,
            _ if id_single == 'A' => Model::AirPodsMax, //single
            _ if id_single == 'B' => Model::PowerbeatsPro,
            "0520" => Model::BeatsX,                      //single
            "1020" => Model::BeatsFlex,                   //single
            "0620" => Model::BeatsSolo3,                  //single
            _ if id_single == '9' => Model::BeatsStudio3, //single
            "0320" => Model::Powerbeats3,                 //single
            _ => Model::Unknown,
        }
    }
}

fn to_hex_str(data: &Packet) -> String {
    data.iter().map(|byte| format!("{:02X}", byte)).collect()
}

fn is_swapped(raw: &str) -> bool {
    let status_byte = raw.chars().nth(10).unwrap().to_string();
    let status_int = i32::from_str_radix(&status_byte, 16).unwrap();
    (status_int & 0x02) == 0
}

#[derive(Debug, Clone)]
pub struct Device {
    pub battery: u8,
    pub charging: bool,
    pub on_ear: Option<bool>,
}

impl Device {
    fn new(raw_battery: i32, charging: bool, on_ear: Option<bool>) -> Self {
        Self {
            battery: (raw_battery * 10) as u8,
            charging,
            on_ear,
        }
    }
}

fn build_devices(hex: &str) -> [Device; 3] {
    // Parsing battery status with hexadecimal base
    let char = &hex.chars().nth(13).unwrap().to_string();
    let pod1_battery = i32::from_str_radix(char, 16).unwrap();

    let char = &hex.chars().nth(12).unwrap().to_string();
    let pod2_battery = i32::from_str_radix(char, 16).unwrap();

    let char = &hex.chars().nth(15).unwrap().to_string();
    let case_battery = i32::from_str_radix(char, 16).unwrap();

    //// Parsing charge status
    let char = &hex.chars().nth(14).unwrap().to_string();
    let charge_bits = i32::from_str_radix(char, 16).unwrap();
    // Determining charging with bitwise operations
    let charge_pod1 = (charge_bits & 0b00000001) != 0;
    let charge_pod2 = (charge_bits & 0b00000010) != 0;
    let charge_case = (charge_bits & 0b00000100) != 0;

    //// Parsing in-ear status
    let char = &hex.chars().nth(11).unwrap().to_string();
    let in_ear_bits = i32::from_str_radix(char, 16).unwrap();
    // Determining in-ear status with bitwise operations
    let in_ear_pod1 = (in_ear_bits & 0b00000010) != 0;
    let in_ear_pod2 = (in_ear_bits & 0b00001000) != 0;

    let mut device1 = Device::new(pod1_battery, charge_pod1, Some(in_ear_pod1));
    let mut device2 = Device::new(pod2_battery, charge_pod2, Some(in_ear_pod2));
    let case = Device::new(case_battery, charge_case, None);

    // If left and right pods are backwards, swap them
    if is_swapped(hex) {
        std::mem::swap(&mut device1, &mut device2);
    }

    [device1, device2, case]
}
