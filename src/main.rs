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

    #[arg(long)]
    get_volume: bool,

    #[arg(long)]
    index: Option<u32>,

    #[arg(long)]
    name: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut pulse = Pulse::connect_to_pulse().unwrap();
    pulse.sync();

    if let Some(inc) = cli.increase {
        pulse.increase_sink_volume(inc, cli.name, cli.index);
    } else if let Some(inc) = cli.decrease {
        pulse.decrease_sink_volume(inc, cli.name, cli.index);
    } else if cli.toggle_mute {
        pulse.toggle_mute(cli.name, cli.index);
    }

    if cli.print_sources {
        pulse.print_sources();
    }

    if cli.print_sinks {
        pulse.print_sinks();
    }

    if cli.get_volume {
        pulse.print_sink_volume(None)
    }
}
