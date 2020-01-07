use super::*;
use std::{env, fmt::Write, path::PathBuf};

const SEP: &str = separator::STR;

fn std_outputs(s: &str) -> Vec<PathBuf> {
    // `std::env` outputs empty paths but we ignore them.
    env::split_paths(s)
        .filter(|path| !path.is_empty())
        .collect()
}

fn path_env_outputs(s: &str) -> Vec<PathBuf> {
    split(s).map(|path| path.to_owned()).collect()
}

#[test]
fn std_compatible() {
    let a = "/path/to/bin";
    let b = "/\u{007F}\u{0080}\u{07FF}\u{0800}\u{FFFF}\u{10000}\u{10FFFF}/bin";

    #[rustfmt::skip]
    let strings: &[&str] = &[
        "",
        a,
        b,
        SEP,
        &format!("{0}{0}",           SEP),
        &format!("{0}{0}{1}",        SEP, a),
        &format!("{0}{1}{0}",        SEP, a),
        &format!("{1}{0}{0}",        SEP, a),
        &format!("{1}{0}{2}",        SEP, a, b),
        &format!("{0}{1}{0}{2}",     SEP, a, b),
        &format!("{1}{0}{2}{0}",     SEP, a, b),
        &format!("{0}{1}{0}{2}{0}",  SEP, a, b),
        &format!("\"{1}\"{0}{2}",    SEP, a, b),
        &format!("{1}{0}\"{2}\"",    SEP, a, b),
        &format!("\"{1}{0}{2}\"",    SEP, a, b),
        &format!("\"{1}{0}\"{0}{2}", SEP, a, b),
        &format!("{1}{0}\"{0}{2}\"", SEP, a, b),
    ];

    let mut mismatches = String::new();

    for s in strings {
        let path_env = path_env_outputs(s);
        let std = std_outputs(s);
        if path_env != std {
            write!(
                mismatches,
                "\nmismatch for {:?}:\n\
                 \tus:  ({}) {:?}\n\
                 \tstd: ({}) {:?}\n",
                s,
                path_env.len(),
                path_env,
                std.len(),
                std,
            )
            .unwrap();
        }
    }

    if !mismatches.is_empty() {
        panic!(
            "found mismatches between `std` and `path_env`:{}",
            mismatches
        );
    }
}
