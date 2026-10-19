use std::env;
use std::process::ExitCode;
use std::time::Instant;

use hanoi64::TowerOfHanoi;

/// Largest tower the solver supports, matching the 20 ring bits per peg slot.
const MAX_RINGS: u8 = 20;

/// Largest tower for which verbose mode is allowed.
const MAX_VERBOSE_RINGS: u8 = 10;

const USAGE: &str = "usage: hanoi <rings> [-v | --verbose]\n\n  \
    <rings>        number of rings, from 1 to 20\n  \
    -v, --verbose  print every move (up to 10 rings)\n  \
    -h, --help     show this help";

#[derive(Debug, PartialEq)]
enum Command {
    Help,
    Solve { rings: u8, verbose: bool },
}

fn main() -> ExitCode {
    match parse_args(env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Solve { rings, verbose }) => {
            solve(rings, verbose);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let args: Vec<String> = args.into_iter().collect();
    // Checked up front so that help is shown even next to invalid arguments.
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(Command::Help);
    }

    let mut rings = None;
    let mut verbose = false;

    for arg in &args {
        match arg.as_str() {
            "-v" | "--verbose" => verbose = true,
            option if option.starts_with('-') => return Err(format!("unknown option '{option}'")),
            value if rings.is_none() => rings = Some(parse_rings(value)?),
            extra => return Err(format!("unexpected argument '{extra}'")),
        }
    }

    let rings = rings.ok_or("missing number of rings")?;
    if verbose && rings > MAX_VERBOSE_RINGS {
        return Err(format!(
            "verbose mode is limited to {MAX_VERBOSE_RINGS} rings: {rings} rings would print {} moves",
            move_count(rings)
        ));
    }
    Ok(Command::Solve { rings, verbose })
}

fn parse_rings(value: &str) -> Result<u8, String> {
    match value.parse::<u8>() {
        Ok(rings) if (1..=MAX_RINGS).contains(&rings) => Ok(rings),
        _ => Err(format!(
            "number of rings must be between 1 and {MAX_RINGS}, got '{value}'"
        )),
    }
}

fn move_count(rings: u8) -> u32 {
    (1 << rings) - 1
}

fn solve(rings: u8, verbose: bool) {
    let mut tower = TowerOfHanoi::new(rings);
    let start = Instant::now();
    tower.solve(verbose);
    let elapsed = start.elapsed();
    println!(
        "Moved {rings} ring{} from peg 0 to peg 2 in {} moves ({elapsed:.2?}).",
        if rings == 1 { "" } else { "s" },
        move_count(rings)
    );
}
