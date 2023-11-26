use pulse::volume::{ChannelVolumes, Volume, VolumeDB};

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
}
