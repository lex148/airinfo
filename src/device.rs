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

fn is_swapped(raw: &str) -> bool {
    let status_byte = raw.chars().nth(10).unwrap().to_string();
    let status_int = i32::from_str_radix(&status_byte, 16).unwrap();
    (status_int & 0x02) == 0
}

pub(crate) fn build_devices(hex: &str) -> [Device; 3] {
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
