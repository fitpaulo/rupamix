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
    #[command(visible_aliases = ["vol", "v"])]
    #[command(about = "Volume control, use volume --help for more info")]
    Volume {
        #[arg(short, long)]
        #[arg(help = "Allow volume to go past 100; hard capped at 120 currently")]
        boost: bool,

        #[arg(short, long)]
        #[arg(conflicts_with_all =  ["decrease", "toggle_mute", "set"])]
        #[arg(default_value = "0")]
        #[arg(num_args = 0..=1)]
        #[arg(default_missing_value = "5")]
        #[arg(help = "Increase volume by the specified amount, or default if not specified")]
        increase: u8,

        #[arg(short, long)]
        #[arg(conflicts_with_all =  ["increase", "toggle_mute", "set"])]
        #[arg(default_value = "0")]
        #[arg(num_args = 0..=1)]
        #[arg(default_missing_value = "5")]
        #[arg(help = "Decrease volume by the specified amount, or default if not specified")]
        decrease: u8,

        #[arg(short, long)]
        #[arg(conflicts_with_all =  ["increase", "decrease", "set"])]
        #[arg(help = "Mutes if not muted, unmutes if muted")]
        toggle_mute: bool,

        #[arg(short, long)]
        #[arg(conflicts_with_all =  ["increase", "decrease", "toggle_mute"])]
        #[arg(help = "Sets the volume to the specified value")]
        set: Option<u8>,
    },

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
        Commands::Volume {
            boost,
            increase,
            decrease,
            toggle_mute,
            set,
        } => {
            if *increase > 0 {
                pulse.increase_sink_volume(increase, cli.index, cli.name, *boost);
            } else if *decrease > 0 {
                pulse.decrease_sink_volume(decrease, cli.index, cli.name);
            } else if *toggle_mute {
                pulse.toggle_mute(cli.index, cli.name);
            } else if set.is_some() {
                pulse.set_sink_volume(set.unwrap(), *boost, cli.index, cli.name);
            } else {
                println!("No action was specified")
            }
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
