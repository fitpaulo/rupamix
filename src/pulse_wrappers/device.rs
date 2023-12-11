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
