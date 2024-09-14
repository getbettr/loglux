#![no_main]
mod concat;

use std::{
    ffi::CStr,
    fs,
    fs::File,
    io::{Error as IoError, ErrorKind, Write},
    os::{
        linux::net::SocketAddrExt,
        unix::{
            fs::FileExt,
            net::{SocketAddr, UnixListener},
        },
    },
    path::PathBuf,
    process, str,
};

use concat::GhettoConcat;

type Res<T> = Result<T, IoError>;
type Brightness = u32;

const NUM_STEPS: f64 = 100_f64;
const BUFFER_SIZE: usize = 30;

struct Controller {
    path: PathBuf,
    max_b: Brightness,
    b: Brightness,
}

fn int_from_file(p: PathBuf) -> Option<u32> {
    let mut b_buf = [b' '; BUFFER_SIZE];
    let _ = File::open(p).ok().and_then(|f| f.read_at(&mut b_buf, 0).ok());

    str::from_utf8(&b_buf).ok().and_then(|x| x.trim().parse::<u32>().ok())
}

impl Controller {
    fn set_brightness(&mut self, new_b: Brightness) -> Res<()> {
        let mut tee = process::Command::new("sudo")
            .arg("tee")
            .arg(self.path.join("brightness"))
            .stdin(process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = tee.stdin.as_mut() {
            let mut buffer = itoa::Buffer::new();
            stdin.write_all(buffer.format(new_b).as_bytes())?;
        }

        tee.wait()?;
        self.b = new_b;
        Ok(())
    }

    fn notify(&self) -> Res<()> {
        let mut buf = itoa::Buffer::new();
        let proc = GhettoConcat::new(
            "int:value:".as_bytes(),
            buf.format(self.b * 100 / self.max_b).as_bytes(),
        );

        process::Command::new("notify-send")
            .arg(self.name()?)
            .args(["-h", proc.as_str(), "-h", "string:synchronous:volume"])
            .output()?;
        Ok(())
    }

    fn name(&self) -> Result<&str, IoError> {
        self.path.file_name().and_then(|f| f.to_str()).ok_or(IoError::from(ErrorKind::Other))
    }

    fn current_step(&self) -> isize {
        (NUM_STEPS * (self.b.max(1) as f64).log(self.max_b as f64)).round() as _
    }

    fn b_from_step(&self, step_no: isize) -> Brightness {
        (self.max_b as f64).powf(step_no as f64 / NUM_STEPS) as _
    }

    fn step_up(&self) -> Brightness {
        let mut step = self.current_step();
        let mut new_b = self.b;

        while new_b <= self.b {
            step += 1;
            new_b = self.b_from_step(step);
        }
        new_b.min(self.max_b)
    }

    fn step_down(&self) -> Brightness {
        let mut step = self.current_step();
        let mut new_b = self.b;

        while new_b >= self.b && step >= 0 {
            step -= 1;
            new_b = self.b_from_step(step);
        }

        new_b
    }
}

fn best_controller(start_path: &PathBuf) -> Option<Controller> {
    let mut path: Option<PathBuf> = None;
    let mut best_max = 0;

    for entry in fs::read_dir(start_path).ok()?.flatten() {
        let c_path = entry.path();
        if let Some(max_b) = int_from_file(c_path.join("max_brightness")) {
            if max_b > best_max {
                best_max = max_b;
                path = Some(c_path);
            }
        }
    }

    path.and_then(|path| {
        let b_path = path.join("brightness");
        Some(Controller { path, max_b: best_max, b: int_from_file(b_path)? })
    })
}

#[no_mangle]
pub fn main(_argc: i32, _argv: *const *const i8) -> Res<()> {
    #[inline]
    fn bail() {
        eprintln!("pass me either 'up' or 'down'");
        process::exit(1);
    }

    if _argc < 2 {
        bail();
    }

    // there can be only one
    let s = SocketAddr::from_abstract_name("lux_lock".as_bytes())?;
    if UnixListener::bind_addr(&s).is_err() {
        process::exit(2);
    }

    // initialize a pointer to store the current argument
    let mut current_arg = _argv;

    // skip the program name
    current_arg = unsafe { current_arg.offset(1) };
    if current_arg.is_null() {
        bail();
    }

    let mode = unsafe { str::from_utf8_unchecked(CStr::from_ptr(*current_arg).to_bytes()) };
    if mode != "up" && mode != "down" {
        bail();
    }

    if let Some(mut controller) = best_controller(&PathBuf::from("/sys/class/backlight")) {
        let _ = match mode {
            "up" => Some(controller.step_up()),
            "down" => Some(controller.step_down()),
            _ => None,
        }
        .and_then(|b| if controller.b != b { controller.set_brightness(b).ok() } else { None })
        .and_then(|_| controller.notify().ok());
    }

    Ok(())
}
