use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

fn main() {
    let program: Vec<char> = match std::env::args().nth(1) {
        Some(arg) => {
            if arg.ends_with(".b") {
                fs::read_to_string(&arg)
                    .expect("Could not read file")
                    .chars()
                    .collect()
            } else {
                arg.chars().collect()
            }
        }
        None => {
            eprintln!("Usage: brainfuck <program|file.bf>");
            std::process::exit(1);
        }
    };

    let mut tape: Vec<u8> = vec![0];
    let mut cell_index: usize = 0;
    let mut loop_table: HashMap<usize, usize> = HashMap::new();
    let mut loop_stack: Vec<usize> = vec![];

    // Build loop table
    for (ip, &instruction) in program.iter().enumerate() {
        if instruction == '[' {
            loop_stack.push(ip);
        } else if instruction == ']' {
            let idx = loop_stack.pop().expect("Unmatched ]");
            loop_table.insert(idx, ip);
            loop_table.insert(ip, idx);
        }
    }

    let mut input_buffer: Vec<u8> = vec![];
    let mut ip: usize = 0;

    while ip < program.len() {
        match program[ip] {
            '+' => tape[cell_index] = tape[cell_index].wrapping_add(1),
            '-' => tape[cell_index] = tape[cell_index].wrapping_sub(1),
            '<' => cell_index -= 1,
            '>' => {
                cell_index += 1;
                if cell_index == tape.len() {
                    tape.push(0);
                }
            }
            '.' => {
                print!("{}", tape[cell_index] as char);
                io::stdout().flush().unwrap();
            }
            ',' => {
                if input_buffer.is_empty() {
                    let mut line = String::new();
                    io::stdin().read_line(&mut line).unwrap();
                    input_buffer = line.into_bytes();
                }
                tape[cell_index] = input_buffer.remove(0);
            }
            '[' => {
                if tape[cell_index] == 0 {
                    ip = loop_table[&ip];
                }
            }
            ']' => {
                if tape[cell_index] != 0 {
                    ip = loop_table[&ip];
                }
            }
            _ => {} // ignore non-BF characters
        }
        ip += 1;
    }
}
