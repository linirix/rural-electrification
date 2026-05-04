fn main() {
    let game = match action_from_args(std::env::args().skip(1)) {
        Ok(CliAction::Run(game)) => *game,
        Ok(CliAction::Help) => {
            print_usage();
            return;
        }
        Err(message) => {
            eprintln!("{message}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    };

    if let Err(error) = electrification::terminal::run_with_game(game) {
        eprintln!("terminal error: {error}");
        std::process::exit(1);
    }
}

enum CliAction {
    Run(Box<electrification::Game>),
    Help,
}

fn action_from_args<I>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = String>,
{
    let mut game_seed = None;
    let mut sandbox = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value after --seed".to_string())?;
                game_seed = Some(parse_seed(&value, "--seed")?);
            }
            "--playtest-seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "missing value after --playtest-seed".to_string())?;
                let playtest_seed = parse_seed(&value, "--playtest-seed")?;
                game_seed = Some(playtest_seed.saturating_mul(1_003));
            }
            "--sandbox" => sandbox = true,
            "-h" | "--help" => return Ok(CliAction::Help),
            _ => return Err(format!("unknown argument '{arg}'")),
        }
    }

    let mut game =
        game_seed.map_or_else(electrification::Game::new, electrification::Game::with_seed);
    if sandbox {
        game.enable_sandbox_mode()
            .map_err(|error| format!("could not enable sandbox mode: {error}"))?;
    }
    Ok(CliAction::Run(Box::new(game)))
}

fn parse_seed(value: &str, flag: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|seed| *seed > 0)
        .ok_or_else(|| format!("{flag} expects a positive integer"))
}

fn print_usage() {
    println!("Electrification: rural utility strategy in the terminal.");
    println!();
    println!("usage: electrification [--seed <game-seed>] [--playtest-seed <seed>] [--sandbox]");
    println!();
    println!("options:");
    println!("  --seed <game-seed>          start a deterministic game seed");
    println!("  --playtest-seed <seed>      start the game seed used by a playtest seed");
    println!("  --sandbox                   disable board review constraints");
}
