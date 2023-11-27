use clap::{Parser, Subcommand};
use rupamix::pulse_wrapper::Pulse;

#[derive(Debug, Parser)]
#[command(name = "Rust Pulse Mixer")]
#[command(author = "Paulo Guimaraes <paulotechusa@proton.me>")]
#[command(version = option_env!("CARGO_PKG_VERSION"))]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(visible_aliases = ["inc", "i"])]
    IncreaseVolume {
        #[arg(short, long, display_order = 0)]
        #[arg(default_value = "5")]
        #[arg(long = "increment")]
        #[arg(help = "The value to increase the volume by, if not specified it uses the default.")]
        inc: u8,

        #[arg(long = "index")]
        #[arg(visible_alias = "idx")]
        #[arg(help = "The index of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "name")]
        idx: Option<u32>,

        #[arg(short, long)]
        #[arg(help = "The name of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "idx")]
        name: Option<String>,
    },

    #[command(visible_aliases = ["dec", "d"])]
    DecreaseVolume {
        #[arg(short, long, display_order = 0)]
        #[arg(default_value = "5")]
        #[arg(long = "increment")]
        #[arg(help = "The value to increase the volume by, if not specified it uses the default.")]
        inc: u8,

        #[arg(long = "index")]
        #[arg(visible_alias = "idx")]
        #[arg(help = "The index of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "name")]
        idx: Option<u32>,

        #[arg(short, long)]
        #[arg(help = "The name of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "idx")]
        name: Option<String>,
    },

    #[command(visible_alias = "t")]
    ToggleMute {
        #[arg(long = "index")]
        #[arg(visible_alias = "idx")]
        #[arg(help = "The index of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "name")]
        idx: Option<u32>,

        #[arg(short, long)]
        #[arg(help = "The name of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "idx")]
        name: Option<String>,
    },

    #[command(visible_alias = "p")]
    Print {
        #[arg(long = "index")]
        #[arg(visible_alias = "idx")]
        #[arg(help = "The index of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "name")]
        idx: Option<u32>,

        #[arg(short, long)]
        #[arg(help = "The name of the Sink. Uses default sink if not specified.")]
        #[arg(conflicts_with = "idx")]
        name: Option<String>,

        #[arg(short, long)]
        sinks: bool,

        #[arg(long)]
        sources: bool,

        #[arg(short, long)]
        volume: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let mut pulse = Pulse::connect_to_pulse().unwrap();
    pulse.sync();

    match &cli.command {
        Commands::Print {
            sinks,
            sources,
            volume,
            idx,
            name,
        } => {
            if *sources {
                pulse.print_sources();
            }

            if *sinks {
                pulse.print_sinks();
            }

            if *volume {
                pulse.print_sink_volume(idx, name)
            }
        }
        Commands::IncreaseVolume { inc, idx, name } => {
            pulse.increase_sink_volume(inc, name, idx);
        }
        Commands::DecreaseVolume { inc, idx, name } => {
            pulse.decrease_sink_volume(inc, name, idx);
        }
        Commands::ToggleMute { idx, name } => pulse.toggle_mute(name, idx),
    }
}
