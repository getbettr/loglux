// To keep our dependency footprint as small as possible we chose not to
// use 'clap' and instead write a manual parser based on 'lexopt'.
use std::path::PathBuf;

use lexopt::{prelude::*, Error};

#[derive(Copy, Clone)]
pub enum Mode {
    Up,
    Down,
}

pub struct Opts {
    // 'up' or 'down'
    pub mode: Mode,

    // Can be either the full path to a controller or the path to a directory
    // containing multiple controllers as sub-directories.
    // In the latter case the "best" controller is chosen as the one that
    // has the maximum 'max_brightness' value.
    pub start_path: PathBuf,

    // Total number of steps.
    pub num_steps: u64,
}

// This is tuned to specifically get 9-10% steps near the range maximum
// and really low values near the minimum.
const DEFAULT_NUM_STEPS: u64 = 75;

// For most linux systems, this is where backlight controllers live.
const DEFAULT_PATH: &str = "/sys/class/backlight";

pub fn help() {
    println!(
        r#"Usage: loglux up|down [-p|--path (default: {})] [-n|--num-steps (default: {:.0})]"#,
        DEFAULT_PATH, DEFAULT_NUM_STEPS
    );
    std::process::exit(0);
}

pub fn parse_opts() -> Result<Opts, Error> {
    let mut mode = Err(Error::from("missing mode"));
    let mut start_path = Ok(PathBuf::from(DEFAULT_PATH));
    let mut num_steps = Ok(DEFAULT_NUM_STEPS);

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) => {
                if val == "up" {
                    mode = Ok(Mode::Up);
                } else if val == "down" {
                    mode = Ok(Mode::Down)
                } else {
                    mode = Err(Error::from(format!("invalid mode: {:?}", val)))
                }
            }
            Short('p') | Long("path") => {
                start_path = parser.value().map(PathBuf::from);
            }
            Short('n') | Long("num-steps") => {
                num_steps = parser.value().and_then(|v| {
                    v.parse::<u64>()
                        .map_err(|e| Error::from(format!("invalid number of steps: {e}")))
                });
            }
            Short('h') | Long("help") => help(),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Opts { mode: mode?, start_path: start_path?, num_steps: num_steps? })
}
