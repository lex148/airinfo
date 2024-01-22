use super::PacketNibble;

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
    pub(crate) fn parse(raw: &PacketNibble) -> Model {
        // Extract the single character and the substring from `status`
        let id_single = raw[7];

        // put the nibbles into a u16
        let id_full: u16 = ((raw[6] as u16 & 0x0F) << 12) | // Low nibble of bytes[0]
            ((raw[7] as u16 & 0x0F) << 8) |                 // Low nibble of bytes[1]
            ((raw[8] as u16 & 0x0F) << 4) |                 // Low nibble of bytes[2]
            (raw[9] as u16 & 0x0F); // Low nibble of bytes[3]

        // Pattern match on `id_full` and `id_single` to identify the pod type
        match id_full {
            0x0220 => Model::AirPods1,
            0x0F20 => Model::AirPods2,
            0x1320 => Model::AirPods3,
            0x0E20 => Model::AirPodsPro,
            0x1420 => Model::AirPodsPro2,
            0x2420 => Model::AirPodsPro2Usbc,
            _ if id_single == 0xA => Model::AirPodsMax, //single
            _ if id_single == 0xB => Model::PowerbeatsPro,
            0x0520 => Model::BeatsX,                      //single
            0x1020 => Model::BeatsFlex,                   //single
            0x0620 => Model::BeatsSolo3,                  //single
            _ if id_single == 0x9 => Model::BeatsStudio3, //single
            0x0320 => Model::Powerbeats3,                 //single
            _ => Model::Unknown,
        }
    }
}
