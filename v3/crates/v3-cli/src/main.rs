use clap::Parser;
use v3_cli::RunError;
use v3_core::config::SimulationConfig;

#[derive(Parser)]
#[command(name = "v3-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Run(RunArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    #[arg(long)]
    ticks: u64,
    #[arg(long, default_value = "1")]
    sample_every: u16,
    #[arg(long)]
    seed: u64,
    #[arg(long)]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(args) => {
            if args.ticks < 1 {
                eprintln!("error: --ticks must be >= 1");
                std::process::exit(1);
            }
            if args.sample_every < 1 {
                eprintln!("error: --sample-every must be >= 1");
                std::process::exit(1);
            }

            let mut config = if let Some(path) = args.config {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: failed to read config file: {e}");
                        std::process::exit(1);
                    }
                };
                match serde_json::from_str::<SimulationConfig>(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("error: invalid config: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                SimulationConfig::default()
            };
            config.normalize();

            let mut out = std::io::stdout();
            match v3_cli::run_simulation(config, args.seed, args.ticks, args.sample_every, &mut out)
            {
                Ok(()) => {}
                Err(RunError::ValidationError(msg)) => {
                    eprintln!("validation error: {msg}");
                    std::process::exit(1);
                }
                Err(RunError::RuntimeError(msg)) => {
                    eprintln!("runtime error: {msg}");
                    std::process::exit(1);
                }
            }
        }
    }
}
