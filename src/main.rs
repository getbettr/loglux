mod cli;
mod controller;
mod stepper;

use std::{
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener},
    },
    process,
};

use cli::*;
use controller::Controller;
use stepper::{Bounded, Stepper};

type LuxErr = Box<dyn std::error::Error + Send + Sync + 'static>;
type LuxRes<T> = Result<T, LuxErr>;

pub fn main() -> LuxRes<()> {
    let opts = parse_opts().unwrap_or_else(|e| {
        eprintln!("error parsing arguments: {}", e);
        help();
        process::exit(1);
    });

    let controller = match Controller::from_opts(&opts) {
        Some(c) => c,
        None => {
            eprintln!("could not find any controller under {}", &opts.start_path.display());
            process::exit(1)
        }
    };

    let new_brightness = match opts.mode {
        Mode::Up => controller.step_up(),
        Mode::Down => controller.step_down(),
    };

    if new_brightness == controller.current() {
        return Ok(());
    }

    if controller.set_brightness(new_brightness).is_ok() {
        // process-level lock the notifications so we don't spam
        let s = SocketAddr::from_abstract_name("loglux_lock".as_bytes())?;
        if UnixListener::bind_addr(&s).is_ok() {
            controller.notify(new_brightness)?;
        }
    }

    Ok(())
}
