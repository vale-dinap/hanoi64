//! Command-line front end for the [`hanoi64`] Tower of Hanoi solver.
//!
//! ```text
//! hanoi64 <rings> [-v | --verbose]
//! ```
//!
//! Solves a tower of `<rings>` rings (1 to 20) and reports the number of moves.
//! With `--verbose`, every move is printed as well.

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use hanoi64::TowerOfHanoi;

/// Largest tower the solver supports, matching the 20 ring bits per peg slot.
const MAX_RINGS: u8 = 20;

/// Largest tower for which verbose mode is allowed.
///
/// `n` rings take `2^n - 1` moves and verbose mode prints one line per move,
/// so the output doubles with every extra ring. 10 rings print 1023 lines,
/// which still fits in a terminal's scrollback; 20 rings would print over a
/// million.
const MAX_VERBOSE_RINGS: u8 = 10;

const USAGE: &str = "usage: hanoi64 <rings> [-v | --verbose]\n\n  \
    <rings>        number of rings, from 1 to 20\n  \
    -v, --verbose  print every move (up to 10 rings)\n  \
    -h, --help     show this help";

/// What the command line asked for.
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

/// Parses the command-line arguments, without the program name.
///
/// Rejects a missing, malformed or out-of-range ring count, unknown options,
/// and verbose mode on more than [`MAX_VERBOSE_RINGS`] rings.
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

/// Parses a ring count, accepting only values from 1 to [`MAX_RINGS`].
fn parse_rings(value: &str) -> Result<u8, String> {
    match value.parse::<u8>() {
        Ok(rings) if (1..=MAX_RINGS).contains(&rings) => Ok(rings),
        _ => Err(format!(
            "number of rings must be between 1 and {MAX_RINGS}, got '{value}'"
        )),
    }
}

/// Number of moves in the optimal solution for `rings` rings: `2^rings - 1`.
fn move_count(rings: u8) -> u32 {
    (1 << rings) - 1
}

/// Solves a tower of `rings` rings and prints a summary.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn parses_rings_and_verbose_flag_in_any_order() {
        assert_eq!(
            parse(&["5"]),
            Ok(Command::Solve {
                rings: 5,
                verbose: false
            })
        );
        assert_eq!(
            parse(&["5", "-v"]),
            Ok(Command::Solve {
                rings: 5,
                verbose: true
            })
        );
        assert_eq!(
            parse(&["--verbose", "5"]),
            Ok(Command::Solve {
                rings: 5,
                verbose: true
            })
        );
    }

    #[test]
    fn help_wins_over_everything_else() {
        assert_eq!(parse(&["-h"]), Ok(Command::Help));
        assert_eq!(parse(&["99", "--help"]), Ok(Command::Help));
    }

    #[test]
    fn rejects_bad_ring_counts() {
        for bad in ["0", "21", "256", "-3", "abc", ""] {
            assert!(parse(&[bad]).is_err(), "accepted '{bad}'");
        }
        assert!(parse(&[]).is_err());
    }

    #[test]
    fn rejects_unknown_options_and_extra_arguments() {
        assert!(parse(&["5", "--fast"]).is_err());
        assert!(parse(&["5", "6"]).is_err());
    }

    #[test]
    fn verbose_is_limited_to_max_verbose_rings() {
        assert!(parse(&[&MAX_VERBOSE_RINGS.to_string(), "-v"]).is_ok());
        assert!(parse(&[&(MAX_VERBOSE_RINGS + 1).to_string(), "-v"]).is_err());
        assert!(parse(&[&MAX_RINGS.to_string()]).is_ok());
    }

    #[test]
    fn move_count_is_two_to_the_n_minus_one() {
        assert_eq!(move_count(1), 1);
        assert_eq!(move_count(3), 7);
        assert_eq!(move_count(20), 1_048_575);
    }
}
