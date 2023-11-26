use pulse::volume::{ChannelVolumes, Volume, VolumeDB};
use std::{
    fs,
    io::{Error, ErrorKind, Read, Write},
};

static FILE: &str = "/tmp/rupamix_vol";
static APPROX_ONE_PCT: VolumeDB = VolumeDB(-120.0);

pub struct SinkSource {
    index: u32,
    name: String,
    volume: ChannelVolumes,
    mute: bool,
}

impl SinkSource {
    pub fn new(index: u32, name: String, volume: ChannelVolumes, mute: bool) -> SinkSource {
        SinkSource {
            index,
            name,
            volume,
            mute,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mute(&self) -> bool {
        self.mute
    }

    pub fn volume(&self) -> ChannelVolumes {
        self.volume
    }

    pub fn increase_volume(&mut self, inc: u8) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;

        while current < initial + inc {
            if current >= 100 {
                println!("Volume is at 100 or above: ({})", current);
                println!("Can't increase any more.");
                break;
            } else {
                self.volume.increase(Volume::from(APPROX_ONE_PCT));
                current = self.get_volume_as_pct();
            }
        }
    }

    pub fn decrease_volume(&mut self, inc: u8) {
        let initial = self.get_volume_as_pct();
        let mut current = initial;

        while current > initial - inc {
            if current == 0 {
                println!("Volume is at 0.");
                println!("Can't decrease any more.");
                break;
            } else {
                self.volume.decrease(Volume::from(APPROX_ONE_PCT));
                current = self.get_volume_as_pct();
            }
        }
    }

    pub fn print_volume(&self) {
        let vols = self.volume.get();
        for vol in vols {
            println!("{}", vol.print());
        }
    }

    fn get_volume_as_pct(&self) -> u8 {
        let vol_str = self.volume.get()[0].print();
        for part in vol_str.split('%') {
            if let Ok(num) = part.trim().parse::<u8>() {
                return num;
            }
        }
        255
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
