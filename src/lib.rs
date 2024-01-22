use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use btleplug::Error;
use std::time::Duration;
use tokio::time;

mod device;
mod model;
mod pod;

pub use device::Device;
pub use model::Model;
pub use pod::Pod;

const APPLE: u16 = 76;
const DATA_LENGTH: usize = 27;
type Packet = [u8; 27];
type PacketNibble = [u8; 54]; /* WARNING: this is really [u4; 54] */
const MIN_RSSI: i16 = -90;

/// Searches for airpods
/// This is the starting point for this library
/// you most likely want to call this to get info about the pods
///
/// To use this library, you must be paired and connected to your headphones.
pub async fn find_pods() -> Result<Vec<Pod>, Error> {
    let bytes = read_bytes().await?;
    Ok(bytes.iter().map(Pod::parse).collect())
}

async fn read_bytes() -> Result<Vec<PacketNibble>, Error> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    let mut found = Vec::default();

    if adapter_list.is_empty() {
        log::warn!("No Bluetooth adapters found");
    }

    for adapter in adapter_list.iter() {
        adapter.start_scan(ScanFilter::default()).await?;
        // We need to wait for the airpods to respond.
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
                    log::debug!("AirPod found");
                    let d: Packet = data.clone().try_into().unwrap();
                    let d = split_u8_to_u4_array(&d);
                    found.push(d);
                }
            }
        }
    }
    Ok(found)
}

/// Take the raw byte string and split it into an array of nibbles.
/// Everything it nibble based not byte based.
fn split_u8_to_u4_array(input: &Packet) -> PacketNibble {
    let mut output = [0u8; 54]; // Initialize output array
    for (i, &byte) in input.iter().enumerate() {
        // High 4 bits
        output[i * 2] = byte >> 4;
        // Low 4 bits
        output[i * 2 + 1] = byte & 0x0F;
    }
    output
}
