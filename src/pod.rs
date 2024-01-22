use super::device::build_devices;
use super::device::Device;
use super::model::Model;
use crate::PacketNibble;

#[derive(Debug, Clone)]
pub struct Pod {
    /// What models of Pod was Found
    pub model: Model,
    /// Left Earbud, will be None if disconnected.
    pub left: Option<Device>,
    /// Right Earbud, will be None if disconnected.
    pub right: Option<Device>,
    /// The Case, Will be None if disconnected or the Pod doesn't have a Case
    pub case: Option<Device>,
}

impl Pod {
    pub(crate) fn parse(raw: &PacketNibble) -> Pod {
        let model = Model::parse(raw);
        let [left_raw, right_raw, case_raw] = build_devices(raw);
        let mut left = Some(left_raw.clone());
        let mut right = Some(right_raw.clone());
        let mut case = Some(case_raw.clone());

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

        if left_raw.battery == 150 {
            left = None
        }
        if right_raw.battery == 150 {
            right = None
        }
        if case_raw.battery == 150 {
            case = None
        }

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
