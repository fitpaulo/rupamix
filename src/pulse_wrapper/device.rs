use pulse::volume::{ChannelVolumes, Volume, VolumeDB};
use std::fs;
use std::io::{Error, ErrorKind, Read, Write};

static MAX_VOLUME: u8 = 120;
static FILE: &str = "/tmp/rupamix_vol";
static APPROX_ONE_PCT: VolumeDB = VolumeDB(-120.0);

pub struct Device {
    index: u32,
    name: String,
    volume: ChannelVolumes,
}

impl Device {
    pub fn new(index: u32, name: String, volume: ChannelVolumes) -> Device {
        Device {
            index,
            name,
            volume,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn volume(&self) -> ChannelVolumes {
        self.volume
    }

    pub fn increase_volume(&mut self, inc: &u8, boost: bool) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;
        // We don't saturate here because we only want a number as big as MAX_VOLUME
        let new_vol = initial.checked_add(*inc).unwrap_or(MAX_VOLUME);

        while current < new_vol {
            if current >= 100 && !boost {
                println!("Volume is at {} use --boost flag to go higher", current);
                break;
            } else {
                self.volume.increase(Volume::from(APPROX_ONE_PCT));
                current = self.get_volume_as_pct();
            }
        }
    }

    pub fn decrease_volume(&mut self, inc: &u8) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;
        // Stop at the bounds
        let new_vol = initial.saturating_sub(*inc);

        // We already know that the smallest the number can get to is zero, no checks needed.
        while current > new_vol {
            self.volume.decrease(Volume::from(APPROX_ONE_PCT));
            current = self.get_volume_as_pct();
        }
    }

    pub fn print_volume(&self) {
        let vols = self.volume.get();
        println!();
        println!("The current volume is: {}", vols[0].print());
    }

    // Made this pub for testing pulse_wrapper
    pub fn get_volume_as_pct(&self) -> u8 {
        let vol_str = self.volume.get()[0].print();
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

    pub fn toggle_mute(&mut self) -> std::io::Result<()> {
        let channels = self.volume.len();

        if self.volume.is_muted() {
            let vol_db = VolumeDB(self.read_tmp_vol()?);
            self.volume.set(channels, Volume::from(vol_db));
        } else {
            let mut file = fs::File::create(FILE)?;
            let current_vol = self.volume.print_db();
            file.write_all(current_vol.as_bytes())?;
            self.volume.mute(channels);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static APPRROX_54_PCT: VolumeDB = VolumeDB(-16.01);

    fn setup() -> Device {
        let mut volume = ChannelVolumes::default();
        volume.set(2_u8, Volume::from(APPRROX_54_PCT));
        Device {
            index: 150,
            name: String::from("Dummy"),
            volume,
            mute: false,
        }
    }

    #[test]
    fn test_get_vol_as_pct() {
        let sink = setup();
        let result = sink.get_volume_as_pct();

        assert_eq!(54_u8, result);
    }

    // These pct I'm using ~54 and ~1 are just that. So we should test many values to know if we need to reduce
    // thet DBs of ~1pct. It should work fine if ~1% is less than 1%
    // Note: we are not counting the loops to see if we are using extra loops to get us to our final place, maybe we
    // should.
    #[test]
    fn test_increase_vol_by_5() {
        let mut sink = setup();
        sink.increase_volume(&5, false);

        assert_eq!(59, sink.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_10() {
        let mut sink = setup();
        sink.increase_volume(&10, false);

        assert_eq!(64, sink.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_15() {
        let mut sink = setup();
        sink.increase_volume(&15, false);

        assert_eq!(69, sink.get_volume_as_pct());
    }

    #[test]
    fn test_increase_vol_by_20() {
        let mut sink = setup();
        sink.increase_volume(&20, false);

        assert_eq!(74, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_5() {
        let mut sink = setup();
        sink.decrease_volume(&5);

        assert_eq!(49, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_10() {
        let mut sink = setup();
        sink.decrease_volume(&10);

        assert_eq!(44, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_15() {
        let mut sink = setup();
        sink.decrease_volume(&15);

        assert_eq!(39, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_20() {
        let mut sink = setup();
        sink.decrease_volume(&20);

        assert_eq!(34, sink.get_volume_as_pct());
    }

    // Lets test the two extremes
    #[test]
    fn test_increase_vol_by_46() {
        let mut sink = setup();
        sink.increase_volume(&46, false);

        assert_eq!(100, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_54() {
        let mut sink = setup();
        sink.decrease_volume(&54);

        assert_eq!(0, sink.get_volume_as_pct());
    }

    // These should give the same results as the last two
    #[test]
    fn test_increase_vol_by_56() {
        let mut sink = setup();
        sink.increase_volume(&56, false);

        assert_eq!(100, sink.get_volume_as_pct());
    }

    #[test]
    fn test_decrease_vol_by_64() {
        let mut sink = setup();
        sink.decrease_volume(&64);

        assert_eq!(0, sink.get_volume_as_pct());
    }

    // Now lets boost shizz!
    #[test]
    fn test_increase_vol_by_50() {
        let mut sink = setup();
        sink.increase_volume(&50, true);

        assert_eq!(104, sink.get_volume_as_pct());
    }

    // Too the max!
    #[test]
    fn test_increase_vol_by_66() {
        let mut sink = setup();
        sink.increase_volume(&66, true);

        assert_eq!(120, sink.get_volume_as_pct());
    }

    // Lets overflow it!
    #[test]
    fn test_increase_vol_by_255() {
        let mut sink = setup();
        sink.increase_volume(&255, true);

        assert_eq!(120, sink.get_volume_as_pct());
    }
}
