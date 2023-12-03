use crate::pulse_controller::device_wrapper::Device;
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

#[cfg(test)]
mod tests {
    use super::*;
    use pulse::volume::VolumeDB;
    static APPRROX_54_PCT: VolumeDB = VolumeDB(-16.01);

    fn setup() -> PulseSourceInfo {
        let name = String::from("test");
        let index = 150;
        let description = String::from("this is a desc");
        let base_volume = Rc::new(RefCell::new(Volume::from(VolumeDB(-16.1))));
        let mut volume = ChannelVolumes::default();
        volume.set(2_u8, Volume::from(APPRROX_54_PCT));
        let volume = Rc::new(RefCell::new(volume));
        PulseSourceInfo {
            name,
            index,
            description,
            volume,
            base_volume,
        }
    }

    #[test]
    fn test_get_vol_as_pct() {
        let source = setup();
        let result = source.get_volume_as_pct();

        assert_eq!(54_u8, result);
    }

    // These pct I'm using ~54 and ~1 are just that. So we should test many values to know if we need to reduce
    // thet DBs of ~1pct. It should work fine if ~1% is less than 1%
    // Note: we are not counting the loops to see if we are using extra loops to get us to our final place, maybe we
    // should.
    #[test]
    fn test_increase_vol_by_5() {
        let mut source = setup();
        source.increase_volume(&5, false);

        assert_eq!(59, source.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_10() {
        let mut source = setup();
        source.increase_volume(&10, false);

        assert_eq!(64, source.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_15() {
        let mut source = setup();
        source.increase_volume(&15, false);

        assert_eq!(69, source.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_20() {
        let mut source = setup();
        source.increase_volume(&20, false);

        assert_eq!(74, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_5() {
        let mut source = setup();
        source.decrease_volume(&5);

        assert_eq!(49, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_10() {
        let mut source = setup();
        source.decrease_volume(&10);

        assert_eq!(44, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_15() {
        let mut source = setup();
        source.decrease_volume(&15);

        assert_eq!(39, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_20() {
        let mut source = setup();
        source.decrease_volume(&20);

        assert_eq!(34, source.get_volume_as_pct());
    }

    // Lets test the two extremes
    #[test]
    fn test_increase_vol_by_46() {
        let mut source = setup();
        source.increase_volume(&46, false);

        assert_eq!(100, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_54() {
        let mut source = setup();
        source.decrease_volume(&54);

        assert_eq!(0, source.get_volume_as_pct());
    }

    // These should give the same results as the last two
    #[test]
    fn test_increase_vol_by_56() {
        let mut source = setup();
        source.increase_volume(&56, false);

        assert_eq!(100, source.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_64() {
        let mut source = setup();
        source.decrease_volume(&64);

        assert_eq!(0, source.get_volume_as_pct());
    }

    // Now lets boost shizz!
    #[test]
    fn test_increase_vol_by_50() {
        let mut source = setup();
        source.increase_volume(&50, true);

        assert_eq!(104, source.get_volume_as_pct());
    }

    // Too the max!
    #[test]
    fn test_increase_vol_by_66() {
        let mut source = setup();
        source.increase_volume(&66, true);

        assert_eq!(120, source.get_volume_as_pct());
    }

    // Lets overflow it!
    #[test]
    fn test_increase_vol_by_255() {
        let mut source = setup();
        source.increase_volume(&255, true);

        assert_eq!(120, source.get_volume_as_pct());
    }
}
