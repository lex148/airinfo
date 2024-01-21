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
    pub(crate) fn parse(hex: &str) -> Model {
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
