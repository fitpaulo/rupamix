use pulse::volume::{ChannelVolumes, Volume, VolumeDB};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs;
use std::io::{Error, ErrorKind, Read, Write};
use std::rc::Rc;

pub static MAX_VOLUME: u8 = 100;
pub static MAX_VOLUME_BOOSTED: u8 = 120;
static FILE: &str = "/tmp/rupamix_vol";
pub static APPROX_ONE_PCT: VolumeDB = VolumeDB(-120.0);

pub trait Device<T> {
    fn index(&self) -> u32;
    fn name(&self) -> &str;
    fn volume(&self) -> Rc<RefCell<ChannelVolumes>>;
    fn base_volume(&self) -> Rc<RefCell<Volume>>;
    fn description(&self) -> &str;

    fn increase_volume(&mut self, inc: &u8, boost: bool) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;
        // We don't saturate here because we only want a number as big as MAX_VOLUME_BOOSTED
        let mut new_vol = initial.checked_add(*inc).unwrap_or(MAX_VOLUME_BOOSTED);

        if boost {
            if new_vol > MAX_VOLUME_BOOSTED {
                new_vol = MAX_VOLUME_BOOSTED;
            }
        } else if new_vol > MAX_VOLUME {
            new_vol = MAX_VOLUME;
        }

        while current < new_vol {
            self.volume()
                .borrow_mut()
                .increase(Volume::from(APPROX_ONE_PCT));
            current = self.get_volume_as_pct();
        }
    }

    fn decrease_volume(&mut self, inc: &u8) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;
        // Stop at the bounds
        let new_vol = initial.saturating_sub(*inc);

        // We already know that the smallest the number can get to is zero, no checks needed.
        while current > new_vol {
            self.volume()
                .borrow_mut()
                .decrease(Volume::from(APPROX_ONE_PCT));
            current = self.get_volume_as_pct();
        }
    }

    #[allow(clippy::comparison_chain)]
    fn set_volume(&mut self, vol: u8, boost: bool) {
        let current_vol = self.get_volume_as_pct();

        match vol.cmp(&current_vol) {
            Ordering::Less => {
                let inc = current_vol - vol;
                self.decrease_volume(&inc);
            }
            Ordering::Equal => {
                println!("\nThe current volume is aleardy {vol}");
            }
            Ordering::Greater => {
                let inc = vol - current_vol;
                self.increase_volume(&inc, boost);
            }
        }
    }

    fn print_volume(&self) {
        let vol = self.volume().borrow_mut().get()[0];
        println!("\nThe current volume is: {}", vol.print());
    }

    // Made this for testing pulse_controller
    fn get_volume_as_pct(&self) -> u8 {
        let vol_str = self.volume().borrow_mut().get()[0].print();
        for part in vol_str.split('%') {
            if let Ok(num) = part.trim().parse::<u8>() {
                return num;
            }
        }
        255_u8
    }

    fn read_tmp_vol(&self) -> std::io::Result<f64> {
        let mut file = fs::File::open(FILE)?;
        let mut vol_str = String::from("");

        file.read_to_string(&mut vol_str)?;

        for item in vol_str.split(' ') {
            if let Ok(num) = item.parse::<f64>() {
                return Ok(num);
            }
        }
        Err(Error::from(ErrorKind::InvalidData))
    }

    fn toggle_mute(&mut self) -> std::io::Result<()> {
        let channels = self.volume().borrow_mut().len();

        if self.volume().borrow_mut().is_muted() {
            let vol_db = VolumeDB(self.read_tmp_vol()?);
            self.volume()
                .borrow_mut()
                .set(channels, Volume::from(vol_db));
        } else {
            let mut file = fs::File::create(FILE)?;
            let current_vol = self.volume().borrow_mut().print_db();
            file.write_all(current_vol.as_bytes())?;
            self.volume().borrow_mut().mute(channels);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulse::volume::VolumeDB;
    use std::cell::RefCell;
    use std::rc::Rc;
    static APPRROX_54_PCT: VolumeDB = VolumeDB(-16.01);

    struct MockDev {
        volume: Rc<RefCell<ChannelVolumes>>,
        base_volume: Rc<RefCell<Volume>>,
    }

    impl Device<MockDev> for MockDev {
        fn index(&self) -> u32 {
            1
        }

        fn name(&self) -> &str {
            "Mock Device"
        }

        fn volume(&self) -> Rc<RefCell<ChannelVolumes>> {
            self.volume.clone()
        }

        fn base_volume(&self) -> Rc<RefCell<Volume>> {
            self.base_volume.clone()
        }

        fn description(&self) -> &str {
            "Description"
        }
    }

    fn setup() -> MockDev {
        let base_volume = Rc::new(RefCell::new(Volume::from(VolumeDB(-16.1))));
        let mut volume = ChannelVolumes::default();
        volume.set(2_u8, Volume::from(APPRROX_54_PCT));
        let volume = Rc::new(RefCell::new(volume));
        MockDev {
            volume,
            base_volume,
        }
    }

    #[test]
    fn test_get_vol_as_pct() {
        let mock_dev = setup();
        let result = mock_dev.get_volume_as_pct();

        assert_eq!(54_u8, result);
    }

    // These pct I'm using ~54 and ~1 are just that. So we should test many values to know if we need to reduce
    // thet DBs of ~1pct. It should work fine if ~1% is less than 1%
    // Note: we are not counting the loops to see if we are using extra loops to get us to our final place, maybe we
    // should.
    #[test]
    fn test_increase_vol_by_5() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&5, false);

        assert_eq!(59, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_10() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&10, false);

        assert_eq!(64, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_15() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&15, false);

        assert_eq!(69, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_20() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&20, false);

        assert_eq!(74, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_5() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&5);

        assert_eq!(49, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_10() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&10);

        assert_eq!(44, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_15() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&15);

        assert_eq!(39, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_20() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&20);

        assert_eq!(34, mock_dev.get_volume_as_pct());
    }

    // Lets test the two extremes
    #[test]
    fn test_increase_vol_by_46() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&46, false);

        assert_eq!(MAX_VOLUME, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_54() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&54);

        assert_eq!(0, mock_dev.get_volume_as_pct());
    }

    // These should give the same results as the last two
    #[test]
    fn test_increase_vol_by_56() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&56, false);

        assert_eq!(MAX_VOLUME, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_64() {
        let mut mock_dev = setup();
        mock_dev.decrease_volume(&64);

        assert_eq!(0, mock_dev.get_volume_as_pct());
    }

    // Now lets boost shizz!
    #[test]
    fn test_increase_vol_by_50() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&50, true);

        assert_eq!(104, mock_dev.get_volume_as_pct());
    }

    // Too the max!
    #[test]
    fn test_increase_vol_by_66() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&66, true);

        assert_eq!(MAX_VOLUME_BOOSTED, mock_dev.get_volume_as_pct());
    }

    // Lets overflow it!
    #[test]
    fn test_increase_vol_by_255() {
        let mut mock_dev = setup();
        mock_dev.increase_volume(&255, true);

        assert_eq!(MAX_VOLUME_BOOSTED, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_set_volume_increase() {
        let mut mock_dev = setup();
        let vol = 65;
        let boost = false;

        mock_dev.set_volume(vol, boost);

        assert_eq!(vol, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_set_volume_increase_mega() {
        let mut mock_dev = setup();
        let vol = 150;
        let boost = false;

        mock_dev.set_volume(vol, boost);

        // 100 is max without boost
        assert_eq!(MAX_VOLUME, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_set_volume_increase_mega_boost() {
        let mut mock_dev = setup();
        let vol = 150;
        let boost = false;

        mock_dev.set_volume(vol, boost);

        // 100 is max without boost
        assert_eq!(MAX_VOLUME, mock_dev.get_volume_as_pct());
    }

    #[test]
    fn test_set_volume_decrease() {
        let mut mock_dev = setup();
        let vol = 20;
        let boost = false;

        mock_dev.set_volume(vol, boost);

        assert_eq!(vol, mock_dev.get_volume_as_pct());
    }
}
