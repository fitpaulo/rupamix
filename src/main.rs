use clap::Parser;
use rupamix::pulse_wrapper::Pulse;

#[derive(Parser)]
#[command(name = "Rust Pulse Mixer")]
#[command(author = "Paulo Guimaraes <paulotechusa@proton.me>")]
#[command(version = option_env!("CARGO_PKG_VERSION"))]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    toggle_mute: bool,

    #[arg(short, long)]
    increase: Option<u8>,

    #[arg(short, long)]
    decrease: Option<u8>,

    #[arg(long)]
    print_sources: bool,

    #[arg(long)]
    print_sinks: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut pulse = Pulse::connect_to_pulse().unwrap();
    pulse.sync();

    if cli.verbose >= 2 {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
    } else if cli.verbose == 1 {
        simple_logger::init_with_level(log::Level::Info).unwrap();
    } else {
        simple_logger::init_with_level(log::Level::Error).unwrap();
    }

    if cli.toggle_mute {
        log::info!("Add code to mute.");
        log::debug!("Blah")
    }

    if let Some(_increment) = cli.increase {
        log::info!("Todo!")
    }

    if let Some(_increment) = cli.decrease {
        log::info!("Todo!")
    }

    if cli.print_sources {
        pulse.print_sources();
    }

    if cli.print_sinks {
        pulse.print_sinks();
    }
}
