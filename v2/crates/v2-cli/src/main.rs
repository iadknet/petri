fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let lines = match args.first().map(String::as_str) {
        Some("run") => run_from_args(&args[1..]),
        Some("ablation") => ablation_from_args(&args[1..]),
        _ => {
            eprintln!(
                "usage: {}",
                [
                    "v2-cli run --ticks <u64> --sample-every <u16> --seed <u64>",
                    "v2-cli ablation --ticks <u64> --seed <u64> --preset <name>",
                ]
                .join(" | ")
            );
            std::process::exit(2);
        }
    };

    for line in lines {
        println!("{line}");
    }
}

fn run_from_args(args: &[String]) -> Vec<String> {
    let ticks = parse_u64(args, "--ticks").unwrap_or(100);
    let sample_every = parse_u16(args, "--sample-every").unwrap_or(10);
    let seed = parse_u64(args, "--seed").unwrap_or(0);
    v2_cli::commands::run_simulation(ticks, sample_every, seed)
}

fn ablation_from_args(args: &[String]) -> Vec<String> {
    let ticks = parse_u64(args, "--ticks").unwrap_or(100);
    let seed = parse_u64(args, "--seed").unwrap_or(0);
    let presets = parse_repeatable(args, "--preset");
    v2_cli::commands::run_ablation(ticks, seed, presets)
}

fn parse_u64(args: &[String], flag: &str) -> Option<u64> {
    parse_flag_value(args, flag)?.parse().ok()
}

fn parse_u16(args: &[String], flag: &str) -> Option<u16> {
    parse_flag_value(args, flag)?.parse().ok()
}

fn parse_repeatable(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .collect()
}

fn parse_flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|pair| (pair[0] == flag).then_some(pair[1].as_str()))
}
