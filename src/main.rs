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

    #[arg(short, long = "index")]
    #[arg(visible_alias = "idx")]
    #[arg(help = "The index of the sink; uses default sink if not specified")]
    #[arg(conflicts_with = "name")]
    idx: Option<u32>,

    #[arg(short, long)]
    #[arg(help = "The name of the sink; uses default sink if not specified")]
    #[arg(conflicts_with = "idx")]
    name: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(visible_aliases = ["inc", "i"])]
    IncreaseVolume {
        #[arg(short)]
        #[arg(default_value = "5")]
        #[arg(long = "increment")]
        #[arg(help = "The value to increase the volume by; uses default if not specified")]
        inc: u8,

        #[arg(short, long)]
        #[arg(help = "Allow volume to go past 100; hard capped at 120 currently")]
        boost: bool,
    },

    #[command(visible_aliases = ["dec", "d"])]
    DecreaseVolume {
        #[arg(short)]
        #[arg(default_value = "5")]
        #[arg(long = "increment")]
        #[arg(help = "The value to increase the volume by, if not specified it uses the default")]
        inc: u8,
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
}

fn main() -> Result<(), &'static str> {
    let cli = Cli::parse();

    let mut pulse = Pulse::new()?;
    pulse.sync();

    match &cli.command {
        Commands::Print {
            sinks,
            sources,
            volume,
        } => {
            if *sources {
                pulse.print_sources();
            }

            if *sinks {
                pulse.print_sinks();
            }

            if *volume {
                pulse.print_sink_volume(cli.idx, cli.name)
            }
        }
        Commands::IncreaseVolume { inc, boost } => {
            pulse.increase_sink_volume(inc, cli.name, cli.idx, *boost);
        }
        Commands::DecreaseVolume { inc } => {
            pulse.decrease_sink_volume(inc, cli.name, cli.idx);
        }
        Commands::ToggleMute => pulse.toggle_mute(cli.name, cli.idx),
    }
    Ok(())
}
