use std::{
    env, fs,
    io::{self, Read},
    process,
};

fn usage() {
    eprintln!("Usage: jsonmend [FILE] [--check] [--explain]\nReads stdin when FILE is omitted.");
}

fn main() {
    let mut file = None;
    let mut check = false;
    let mut explain = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--check" => check = true,
            "--explain" => explain = true,
            "-h" | "--help" => {
                usage();
                return;
            }
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                usage();
                process::exit(2);
            }
            value if file.is_none() => file = Some(value.to_string()),
            value => {
                eprintln!("unexpected argument: {value}");
                process::exit(2);
            }
        }
    }
    let input = match file {
        Some(path) => fs::read_to_string(&path).unwrap_or_else(|error| {
            eprintln!("jsonmend: {path}: {error}");
            process::exit(1)
        }),
        None => {
            let mut value = String::new();
            io::stdin()
                .read_to_string(&mut value)
                .unwrap_or_else(|error| {
                    eprintln!("jsonmend: stdin: {error}");
                    process::exit(1)
                });
            value
        }
    };
    let result = jsonmend::repair(&input);
    if explain {
        if result.changes.is_empty() {
            eprintln!("no repairs needed");
        }
        for change in &result.changes {
            eprintln!("- {change}");
        }
    }
    println!("{}", result.output);
    if check && !result.changes.is_empty() {
        process::exit(2);
    }
}
