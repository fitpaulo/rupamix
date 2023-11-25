use clap::Parser;
use rupamix::Pulse;

#[derive(Parser)]
#[command(name = "Rust Pulse Mixer")]
#[command(author = "Paulo Guimaraes <paulotechusa@proton.me>")]
#[command(version = option_env!("CARGO_PKG_VERSION"))]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    toggle_mute: bool,

    #[arg(short, long)]
    increase: Option<u8>,

    #[arg(short, long)]
    decrease: Option<u8>,
}

fn main() {
    let cli = Cli::parse();

    if cli.toggle_mute {
        println!("Add code to mute.")
    }

    if let Some(_increment) = cli.increase {
        let mut pulse = Pulse::connect_to_pulse().unwrap();
        pulse.get_sink_info();
        pulse.update_volume();
    }

    if let Some(_increment) = cli.decrease {
        let mut pulse = Pulse::connect_to_pulse().unwrap();
        pulse.get_sink_info();
        pulse.update_volume();
    }
}
