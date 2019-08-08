// Based on https://github.com/dtolnay/cargo-expand

use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use yarte_config::PrintOption;

pub fn log(s: &str, path: String, option: &PrintOption) {
    if definitely_not_nightly() {
        logger(s, path, option);
    } else {
        writeln!(io::stdout(), "{}", s).unwrap();
    }
}

fn logger(s: &str, path: String, option: &PrintOption) {
    let rustfmt =
        which_rustfmt().expect("Install rustfmt by running `rustup component add rustfmt`.");

    let mut builder = tempfile::Builder::new();
    builder.prefix("yarte");
    let outdir = builder.tempdir().expect("failed to create tmp file");
    let outfile_path = outdir.path().join("expanded");
    fs::write(&outfile_path, s).expect("correct write to file");

    // Ignore any errors.
    let _status = Command::new(rustfmt)
        .arg(&outfile_path)
        .stderr(Stdio::null())
        .status();

    let mut s = fs::read_to_string(&outfile_path).unwrap();
    if option.short.unwrap_or(true) {
        let lines: Vec<&str> = s.lines().collect();
        s = if cfg!(feature = "actix-web") {
            lines[0..lines.len() - 22].join("\n")
        } else {
            lines[0..lines.len() - 10].join("\n")
        };
    }

    println!("{}\n{}", path, s);
}

fn cargo_binary() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| "cargo".to_owned().into())
}

fn definitely_not_nightly() -> bool {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("--version");

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return false,
    };

    let version = match String::from_utf8(output.stdout) {
        Ok(version) => version,
        Err(_) => return false,
    };

    version.starts_with("cargo 1") && !version.contains("nightly")
}

fn which_rustfmt() -> Option<PathBuf> {
    match env::var_os("RUSTFMT") {
        Some(which) => {
            if which.is_empty() {
                None
            } else {
                Some(PathBuf::from(which))
            }
        }
        None => toolchain_find::find_installed_component("rustfmt"),
    }
}
