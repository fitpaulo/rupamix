use crate::pulse_wrappers::device::Device;
use pulse::context::introspect::SourceInfo;
use pulse::volume::{ChannelVolumes, Volume};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PulseSourceInfo {
    name: String,
    index: u32,
    description: String,
    volume: Rc<RefCell<ChannelVolumes>>,
    base_volume: Rc<RefCell<Volume>>,
}

impl PulseSourceInfo {
    pub fn new(
        name: String,
        index: u32,
        description: String,
        volume: Rc<RefCell<ChannelVolumes>>,
        base_volume: Rc<RefCell<Volume>>,
    ) -> PulseSourceInfo {
        PulseSourceInfo {
            name,
            index,
            description,
            volume,
            base_volume,
        }
    }
}

impl From<&'_ SourceInfo<'_>> for PulseSourceInfo {
    fn from(item: &SourceInfo) -> Self {
        PulseSourceInfo {
            name: String::from(item.name.clone().unwrap()),
            index: item.index,
            description: String::from(item.description.clone().unwrap()),
            volume: Rc::new(RefCell::new(item.volume)),
            base_volume: Rc::new(RefCell::new(item.base_volume)),
        }
    }
}

impl Device<PulseSourceInfo> for PulseSourceInfo {
    fn index(&self) -> u32 {
        self.index
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn volume(&self) -> Rc<RefCell<ChannelVolumes>> {
        self.volume.clone()
    }

    fn base_volume(&self) -> Rc<RefCell<Volume>> {
        self.base_volume.clone()
    }

    fn description(&self) -> &str {
        &self.description
    }
}
