use super::PacketNibble;

#[derive(Debug, Clone)]
pub struct Device {
    /// The battery level for this device. 0-100.
    /// NOTE: this is only reported in 10 interval steps
    pub battery: u8,

    /// True if the device is charging.
    pub charging: bool,

    /// true if the device reports in-ear / on-ear.
    /// NOTE: this can be slow to updated
    pub on_ear: Option<bool>,
}

impl Device {
    fn new(raw_battery: u8, charging: bool, on_ear: Option<bool>) -> Self {
        Self {
            battery: raw_battery * 10,
            charging,
            on_ear,
        }
    }
}

fn is_swapped(raw: &PacketNibble) -> bool {
    let status_int = raw[10];
    (status_int & 0x02) == 0
}

pub(crate) fn build_devices(raw: &PacketNibble) -> [Device; 3] {
    // Parsing battery status with hexadecimal base
    let pod1_battery = raw[13];
    let pod2_battery = raw[12];
    let case_battery = raw[15];

    //// Parsing charge status
    let charge_bits = raw[14];
    // Determining charging with bitwise operations
    let charge_pod1 = (charge_bits & 0b0001) != 0;
    let charge_pod2 = (charge_bits & 0b0010) != 0;
    let charge_case = (charge_bits & 0b0100) != 0;

    //// Parsing in-ear status
    let in_ear_bits = raw[11];
    // Determining in-ear status with bitwise operations
    let in_ear_pod1 = (in_ear_bits & 0b0010) != 0;
    let in_ear_pod2 = (in_ear_bits & 0b1000) != 0;

    let mut device1 = Device::new(pod1_battery, charge_pod1, Some(in_ear_pod1));
    let mut device2 = Device::new(pod2_battery, charge_pod2, Some(in_ear_pod2));
    let case = Device::new(case_battery, charge_case, None);

    // If left and right pods are backwards, swap them
    if is_swapped(raw) {
        std::mem::swap(&mut device1, &mut device2);
    }

    [device1, device2, case]
}
