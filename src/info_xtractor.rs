use colored::*;
use pulse::volume::{ChannelVolumes, Volume, VolumeDB, VolumeLinear};

// These two values were determined by the find_one_pct method
// I'm not sure if these are dependent on my machine or if they are universal
// Until I have confirmation, I tend to think these are HW specific
// static GUESS_ONE_PCT_MIDPOINT: VolumeDB = VolumeDB(-124.0);
// static GUESS_ONE_PCT_LARGEST: VolumeDB = VolumeDB(-110.0);

// static GUESS_ONE_PCT_LINEAR_MIDPOINT: VolumeLinear = VolumeLinear(6.3062205e-7);
// static GUESS_ONE_PCT_LINEAR_MIDPOINT: VolumeLinear = VolumeLinear(3.16289973e-6);

#[derive(Default)]
pub struct InfoXtractor {
    ones: Vec<i32>,
    approx_one_pct: Option<Volume>,
    approx_one_pct_db: Option<VolumeDB>,
    approx_one_pct_linear: Option<VolumeLinear>,
}

#[allow(clippy::comparison_chain)]
impl InfoXtractor {
    pub fn new(verbose: u8) -> InfoXtractor {
        let mut info = InfoXtractor::default();
        info.find_ones(verbose);
        info
    }

    pub fn approx_one_pct(&self) -> &Volume {
        self.approx_one_pct.as_ref().unwrap()
    }

    pub fn approx_one_pct_linear(&self) -> &VolumeLinear {
        self.approx_one_pct_linear.as_ref().unwrap()
    }

    pub fn approx_one_pct_db(&self) -> &VolumeDB {
        self.approx_one_pct_db.as_ref().unwrap()
    }

    pub fn ones(&self) -> &[i32] {
        &self.ones
    }

    pub fn find_ones(&mut self, verbose: u8) {
        for i in -160..-90 {
            let vol = Volume::from(VolumeDB(f64::from(i)));
            let pct = read_volume_as_pct(vol);
            if pct == 1 {
                self.ones.push(i);
            } else if pct > 1 {
                break;
            }
        }

        if verbose > 0 {
            println!("Found {} volumes that register as 1%", self.ones.len());
        }
    }

    /// This shound find a "good enough" value for 1%
    pub fn one_pct_midpoint(&mut self, verbose: u8) -> Volume {
        // It turns out the midpoint led to meh results. took 116 steps to go from 1% to 100% volume
        // This value will work based on the way we increas volume
        let midpoint = self.ones[self.ones.len() / 2];

        if verbose > 0 {
            println!("The middle 1% is {}", midpoint);
        }

        Volume::from(VolumeDB(f64::from(midpoint)))
    }

    /// This shound find a "good enough" value for 1%
    pub fn one_pct_largest(&mut self, verbose: u8) -> Volume {
        // It turns out the midpoint led to bad results. took 71 steps to go from 1% to 100% volume
        // This value will work based on the way we increas volume
        let largest = self.ones[self.ones.len() - 1];

        if verbose > 0 {
            println!("The largest 1% is {}", largest);
        }

        Volume::from(VolumeDB(f64::from(largest)))
    }

    pub fn print_approx_one_pct_volumes(&self) {
        println!("Verbose volume info");
        println!("{}", self.approx_one_pct().print_verbose(true));
        println!();

        println!("Verbose volumeLinear info");
        println!("{:?}", self.approx_one_pct_linear());
        println!();
    }

    pub fn test_one_pct_midpoint(&mut self, verbose: u8) {
        let mut count: u8 = 0;
        let one_pct = self.one_pct_midpoint(verbose);
        let mut vol = ChannelVolumes::default();
        vol.set(2, one_pct);
        let starting_pct = read_volume_as_pct(vol.get()[0]);

        loop {
            if read_volume_as_pct(vol.get()[0]) >= 100 {
                break;
            }
            vol.increase(one_pct);
            count += 1;
        }

        let ending_pct = read_volume_as_pct(vol.get()[0]);

        println!();
        println!("Went from {starting_pct}% to {ending_pct}%");
        println!("This process took {count} steps");
    }

    pub fn test_one_pct_largest(&mut self, verbose: u8) {
        let mut count: u8 = 0;
        let one_pct = self.one_pct_largest(verbose);
        let mut vol = ChannelVolumes::default();
        vol.set(2, one_pct);
        let starting_pct = read_volume_as_pct(vol.get()[0]);

        // It turns out the largest led to bad results. took 67 steps to go from 1% to 100% volume
        // This value will not work with our program
        loop {
            if read_volume_as_pct(vol.get()[0]) >= 100 {
                break;
            }
            vol.increase(one_pct);
            count += 1;
        }

        let ending_pct = read_volume_as_pct(vol.get()[0]);

        println!();
        println!("Went from {starting_pct}% to {ending_pct}%");
        println!("This process took {count} steps");
    }

    pub fn test_one_pct_better(&self) {
        for one in self.ones() {
            let mut count: u16 = 0;
            let end_vol = 200;
            let mut vol = ChannelVolumes::default();
            let one_pct = Volume::from(VolumeDB(f64::from(*one)));
            vol.set(2, one_pct);
            let starting_pct = read_volume_as_pct(vol.get()[0]);
            let mut output: String = String::new();

            // It turns out the largest led to bad results. took 67 steps to go from 1% to 100% volume
            // This value will not work with our program
            loop {
                if read_volume_as_pct(vol.get()[0]) >= end_vol {
                    break;
                }
                vol.increase(one_pct);
                count += 1;
            }

            let ending_pct = read_volume_as_pct(vol.get()[0]);

            output.push_str("\nFor 'one': ");
            output.push_str(&one.to_string());
            output.push_str("dB\n");
            output.push_str("Volume went from ");
            output.push_str(&starting_pct.to_string());
            output.push_str("% to ");
            output.push_str(&ending_pct.to_string());
            output.push_str("%\n");
            output.push_str("This process took ");
            output.push_str(&count.to_string());
            output.push_str("steps.");

            match count.cmp(&u16::from(end_vol - 1)) {
                std::cmp::Ordering::Less => {
                    println!("{}", output.red());
                }
                std::cmp::Ordering::Equal => {
                    println!("{}", output.green());
                }
                std::cmp::Ordering::Greater => {
                    println!("{}", output.yellow());
                }
            }
        }
    }
}

fn read_volume_as_pct(vol: Volume) -> u8 {
    let vol_str = vol.print();
    for part in vol_str.split('%') {
        if let Ok(num) = part.trim().parse::<u8>() {
            return num;
        }
    }
    255_u8
}
