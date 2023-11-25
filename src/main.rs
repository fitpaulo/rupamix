use clap::Parser;
use ffi_learn::Pulse;

#[derive(Parser)]
#[command(name = "Rust PA Mixer")]
#[command(author = "Paulo Guimaraes <paulotechusa@proton.me>")]
#[command(version = option_env!("CARGO_PKG_VERSION"))]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    mute: bool,

    #[arg(short, long)]
    unmute: bool,

    #[arg(long)]
    server: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.mute {
        println!("Add code to mute.")
    }

    if cli.unmute {
        println!("Add code to unmute.")
    }

    if cli.server {
        let mut pulse = Pulse::connect_to_pulse().unwrap();
        pulse.get_sink_info();
        pulse.update_volume();
    }
}
