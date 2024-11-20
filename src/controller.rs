use std::{
    borrow::Cow,
    fs::{read_dir, File, OpenOptions},
    io::{Result as IoResult, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    str,
};

use crate::{cli::Opts, stepper::Bounded, LuxRes};

const BUFFER_SIZE: usize = 32;

#[derive(Clone)]
pub struct Controller<'p> {
    pub path: Cow<'p, PathBuf>,
    max_brightness: u64,
    brightness: u64,
    num_steps: u64,
}

impl Bounded for Controller<'_> {
    fn current(&self) -> u64 { self.brightness }
    fn max(&self) -> u64 { self.max_brightness }
    fn num_steps(&self) -> u64 { self.num_steps }
    fn with_current(&self, brightness: u64) -> Self { Self { brightness, ..self.clone() } }
}

impl<'p> Controller<'p> {
    pub fn from_opts(opts: &'p Opts) -> Option<Self> {
        let mut path = Cow::Borrowed(&opts.start_path);

        // We've been passed the path to a specific controller
        // => read the 'max_brightness' and 'brightness' values and return it.
        if let (Some(max_brightness), Some(brightness)) = (
            val_from_file(opts.start_path.join("max_brightness")),
            val_from_file(opts.start_path.join("brightness")),
        ) {
            return Some(Controller {
                path,
                max_brightness,
                brightness,
                num_steps: opts.num_steps,
            });
        }

        let mut max_brightness = 0;
        let mut found = false;
        let start_path = path.to_mut();

        // Try to recurse into 'start_path' and pick the controller with the
        // maximum 'max_brightness' value.
        for entry in read_dir(&opts.start_path).ok()?.flatten() {
            let c_path = entry.path();
            if let Some(max_b) = val_from_file(c_path.join("max_brightness")) {
                if max_b > max_brightness {
                    max_brightness = max_b;
                    *start_path = c_path;
                    found = true;
                }
            }
        }

        if found {
            let brightness = val_from_file(start_path.join("brightness"))?;
            return Some(Controller {
                path,
                max_brightness,
                brightness,
                num_steps: opts.num_steps,
            });
        }
        None
    }

    pub fn brightness_path(&self) -> PathBuf { self.path.join("brightness") }

    pub fn set_brightness(&self, new_b: u64) -> LuxRes<()> {
        let mut file = OpenOptions::new().write(true).open(self.brightness_path())?;

        file.write_all(format!("{}", new_b).as_bytes()).map_err(|e| e.into())
    }

    pub fn notify(&self, new_b: u64) -> LuxRes<()> {
        let output = Command::new("notify-send")
            .args([
                self.name()?,
                "-h",
                &format!("int:value:{}", new_b * 100 / self.max_brightness),
                "-h",
                "string:synchronous:volume",
            ])
            .output();

        cmd_result("notify-send", output)
    }

    fn name(&self) -> LuxRes<&str> {
        self.path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("could not determine controller name".into())
    }
}

fn cmd_result(cmd_name: &str, output: IoResult<Output>) -> LuxRes<()> {
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = str::from_utf8(&out.stderr).unwrap_or_default().trim();
            Err(format!("{} failed: {}", cmd_name, stderr).into())
        }
        Err(e) => Err(format!("{} failed: {e}", cmd_name).into()),
    }
}

fn val_from_file<V, P>(file: P) -> Option<V>
where
    V: str::FromStr,
    P: AsRef<Path>,
{
    let mut b_buf = [b' '; BUFFER_SIZE];
    let _ = File::open(file).ok().and_then(|f| f.read_at(&mut b_buf, 0).ok());

    str::from_utf8(&b_buf).ok().and_then(|x| x.trim().parse::<V>().ok())
}
