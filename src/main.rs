use clap::{Parser, Subcommand};
use rupamix::pulse_controller::Pulse;

#[cfg(feature = "extractor")]
use rupamix::info_xtractor::InfoXtractor;

#[derive(Debug, Parser)]
#[command(name = "Rust Pulse Mixer")]
#[command(author = "Paulo Guimaraes <paulotechusa@proton.me>")]
#[command(version = option_env!("CARGO_PKG_VERSION"))]
#[command(about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    #[arg(visible_alias = "index")]
    #[arg(help = "The index of the sink; uses default sink if not specified")]
    #[arg(conflicts_with = "name")]
    index: Option<u32>,

    #[arg(short, long)]
    #[arg(help = "The name of the sink; uses default sink if not specified")]
    #[arg(conflicts_with = "index")]
    name: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(visible_aliases = ["inc", "i"])]
    IncreaseVolume {
        #[arg(short, long = "increment")]
        #[arg(default_value = "5")]
        #[arg(help = "The value to increase the volume by; uses default if not specified")]
        inc: u8,

        #[arg(short, long)]
        #[arg(help = "Allow volume to go past 100; hard capped at 120 currently")]
        boost: bool,
    },

    #[command(visible_aliases = ["dec", "d"])]
    DecreaseVolume {
        #[arg(short, long = "increment")]
        #[arg(default_value = "5")]
        #[arg(help = "The value to increase the volume by, if not specified it uses the default")]
        inc: u8,
    },

    #[command(visible_aliases = ["s"])]
    SetVolume {
        #[arg(short, long = "volume")]
        #[arg(help = "The value (0-100) to set the volume to")]
        vol: u8,

        #[arg(short, long)]
        #[arg(help = "Allow volume to go past 100; hard capped at 120 currently")]
        boost: bool,
    },

    #[command(visible_alias = "t")]
    ToggleMute,

    #[command(visible_alias = "p")]
    #[command(about = "Prints various data you may be interested in")]
    Print {
        #[arg(short, long)]
        #[arg(help = "Prints the index and name of all the sinks")]
        sinks: bool,

        #[arg(long)]
        #[arg(help = "Prints the index and name of all the sources")]
        sources: bool,

        #[arg(short, long)]
        #[arg(help = "Prints the volume of the specifed sink or the default if not specified")]
        volume: bool,
    },

    #[command(visible_alias = "x")]
    #[command(
        about = "Gets system info about volumes, really only useful if you are developing this tool"
    )]
    #[cfg(feature = "extractor")]
    Extractor {
        #[arg(short)]
        #[arg(
            help = "Prints a functional approximation of one pct for both dB and Linear Volume structs"
        )]
        one_percent: bool,
    },
}

fn main() -> Result<(), &'static str> {
    let cli = Cli::parse();

    let mut pulse = Pulse::new();

    match &cli.command {
        Commands::Print {
            sinks,
            sources,
            volume,
        } => {
            if *sources {
                pulse.print_sources()
            }

            if *sinks {
                pulse.print_sinks()
            }

            if *volume {
                pulse.print_sink_volume(cli.index, cli.name);
            }
        }
        Commands::IncreaseVolume { inc, boost } => {
            pulse.increase_sink_volume(inc, cli.index, cli.name, *boost);
        }
        Commands::DecreaseVolume { inc } => {
            pulse.decrease_sink_volume(inc, cli.index, cli.name);
        }
        Commands::ToggleMute => pulse.toggle_mute(cli.index, cli.name),
        Commands::SetVolume { vol, boost } => {
            pulse.set_sink_volume(*vol, *boost, cli.index, cli.name)
        }
        #[cfg(feature = "extractor")]
        Commands::Extractor { one_percent } => {
            if *one_percent {
                let xtractor = InfoXtractor::new(cli.verbose);
                // xtractor.test_one_pct_midpoint(cli.verbose);
                // xtractor.test_one_pct_largest(cli.verbose);
                xtractor.test_one_pct_better();
            }
        }
    }
    Ok(())
}
