use std::fs;
use std::io;
use brainfuck_rs::{compile, execute};

fn main() {
    let program: Vec<u8> = match std::env::args().nth(1) {
        Some(arg) => {
            if arg.ends_with(".b") {
                fs::read(&arg).expect("Could not read file")
            } else {
                arg.into_bytes()
            }
        }
        None => {
            eprintln!("Usage: brainfuck <program|file.b>");
            std::process::exit(1);
        }
    };

    let instructions = match compile(&program) {
        Ok(ins) => ins,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = execute(&instructions, io::stdin(), io::stdout()) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
