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
const MIN_RSSI: i16 = -90;

/// Searches for airpods
/// This is the starting point for this library
/// you most likely want to call this to get info about the pods
pub async fn find_pods() -> Result<Vec<Pod>, Error> {
    let bytes = read_bytes().await?;
    Ok(bytes.iter().map(Pod::parse).collect())
}

async fn read_bytes() -> Result<Vec<Packet>, Error> {
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
                    found.push(d);
                }
            }
        }
    }
    Ok(found)
}
