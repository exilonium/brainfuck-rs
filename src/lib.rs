use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    Add(u8),
    Sub(u8),
    MoveRight(usize),
    MoveLeft(usize),
    Output,
    Input,
    JumpIfZero(usize),
    JumpIfNotZero(usize),
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileError {
    UnmatchedOpenBracket,
    UnmatchedCloseBracket,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::UnmatchedOpenBracket => write!(f, "Syntax error: unmatched '['"),
            CompileError::UnmatchedCloseBracket => write!(f, "Syntax error: unmatched ']'"),
        }
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    PointerUnderflow,
    PointerOverflow,
    WriteError,
    ReadError,
    InvalidJumpTarget,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::PointerUnderflow => write!(f, "Runtime error: tape pointer underflow"),
            RuntimeError::PointerOverflow => write!(f, "Runtime error: tape pointer overflow"),
            RuntimeError::WriteError => write!(f, "Runtime error: failed to write output"),
            RuntimeError::ReadError => write!(f, "Runtime error: failed to read input"),
            RuntimeError::InvalidJumpTarget => write!(f, "Runtime error: invalid jump target"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub fn is_bf_char(c: u8) -> bool {
    matches!(c, b'+' | b'-' | b'<' | b'>' | b'.' | b',' | b'[' | b']')
}

pub fn compile(program: &[u8]) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = Vec::new();
    let mut ip = 0;
    
    while ip < program.len() {
        match program[ip] {
            b'+' => {
                let mut count = 0u8;
                while ip < program.len() && program[ip] == b'+' {
                    count = count.wrapping_add(1);
                    ip += 1;
                }
                if count != 0 {
                    instructions.push(Instruction::Add(count));
                }
            }
            b'-' => {
                let mut count = 0u8;
                while ip < program.len() && program[ip] == b'-' {
                    count = count.wrapping_add(1);
                    ip += 1;
                }
                if count != 0 {
                    instructions.push(Instruction::Sub(count));
                }
            }
            b'>' => {
                let mut count = 0;
                while ip < program.len() && program[ip] == b'>' {
                    count += 1;
                    ip += 1;
                }
                if count != 0 {
                    instructions.push(Instruction::MoveRight(count));
                }
            }
            b'<' => {
                let mut count = 0;
                while ip < program.len() && program[ip] == b'<' {
                    count += 1;
                    ip += 1;
                }
                if count != 0 {
                    instructions.push(Instruction::MoveLeft(count));
                }
            }
            b'.' => {
                instructions.push(Instruction::Output);
                ip += 1;
            }
            b',' => {
                instructions.push(Instruction::Input);
                ip += 1;
            }
            b'[' => {
                // Look ahead to check for zeroing loop: [-] or [+]
                let mut next_ip = ip + 1;
                while next_ip < program.len() && !is_bf_char(program[next_ip]) {
                    next_ip += 1;
                }
                if next_ip < program.len() && (program[next_ip] == b'-' || program[next_ip] == b'+') {
                    let mut end_ip = next_ip + 1;
                    while end_ip < program.len() && !is_bf_char(program[end_ip]) {
                        end_ip += 1;
                    }
                    if end_ip < program.len() && program[end_ip] == b']' {
                        instructions.push(Instruction::Clear);
                        ip = end_ip + 1;
                        continue;
                    }
                }
                
                instructions.push(Instruction::JumpIfZero(0));
                ip += 1;
            }
            b']' => {
                instructions.push(Instruction::JumpIfNotZero(0));
                ip += 1;
            }
            _ => {
                ip += 1;
            }
        }
    }
    
    // Resolve jumps
    let mut loop_stack = Vec::new();
    for i in 0..instructions.len() {
        match instructions[i] {
            Instruction::JumpIfZero(_) => {
                loop_stack.push(i);
            }
            Instruction::JumpIfNotZero(_) => {
                let start = loop_stack.pop().ok_or(CompileError::UnmatchedCloseBracket)?;
                instructions[start] = Instruction::JumpIfZero(i + 1);
                instructions[i] = Instruction::JumpIfNotZero(start + 1);
            }
            _ => {}
        }
    }
    if !loop_stack.is_empty() {
        return Err(CompileError::UnmatchedOpenBracket);
    }
    
    Ok(instructions)
}

pub fn execute<R: io::Read, W: io::Write>(
    instructions: &[Instruction],
    mut input: R,
    mut output: W,
) -> Result<(), RuntimeError> {
    let mut tape: Vec<u8> = vec![0; 30000];
    let mut cell_index: usize = 0;
    let mut ip: usize = 0;
    let mut buf = [0; 1];

    while ip < instructions.len() {
        match instructions[ip] {
            Instruction::Add(val) => {
                tape[cell_index] = tape[cell_index].wrapping_add(val);
                ip += 1;
            }
            Instruction::Sub(val) => {
                tape[cell_index] = tape[cell_index].wrapping_sub(val);
                ip += 1;
            }
            Instruction::MoveRight(val) => {
                cell_index = cell_index.checked_add(val).ok_or(RuntimeError::PointerOverflow)?;
                let new_len = cell_index.checked_add(1).ok_or(RuntimeError::PointerOverflow)?;
                if cell_index >= tape.len() {
                    tape.resize(new_len, 0);
                }
                ip += 1;
            }
            Instruction::MoveLeft(val) => {
                if cell_index < val {
                    return Err(RuntimeError::PointerUnderflow);
                }
                cell_index -= val;
                ip += 1;
            }
            Instruction::Output => {
                output.write_all(&[tape[cell_index]]).map_err(|_| RuntimeError::WriteError)?;
                output.flush().map_err(|_| RuntimeError::WriteError)?;
                ip += 1;
            }
            Instruction::Input => {
                match input.read(&mut buf) {
                    Ok(0) => {
                        tape[cell_index] = 0;
                    }
                    Ok(_) => {
                        tape[cell_index] = buf[0];
                    }
                    Err(_) => {
                        return Err(RuntimeError::ReadError);
                    }
                }
                ip += 1;
            }
            Instruction::JumpIfZero(target) => {
                if target > instructions.len() {
                    return Err(RuntimeError::InvalidJumpTarget);
                }
                if tape[cell_index] == 0 {
                    ip = target;
                } else {
                    ip += 1;
                }
            }
            Instruction::JumpIfNotZero(target) => {
                if target > instructions.len() {
                    return Err(RuntimeError::InvalidJumpTarget);
                }
                if tape[cell_index] != 0 {
                    ip = target;
                } else {
                    ip += 1;
                }
            }
            Instruction::Clear => {
                tape[cell_index] = 0;
                ip += 1;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bf_char() {
        assert!(is_bf_char(b'+'));
        assert!(is_bf_char(b'-'));
        assert!(is_bf_char(b'<'));
        assert!(is_bf_char(b'>'));
        assert!(is_bf_char(b'.'));
        assert!(is_bf_char(b','));
        assert!(is_bf_char(b'['));
        assert!(is_bf_char(b']'));
        assert!(!is_bf_char(b'a'));
        assert!(!is_bf_char(b' '));
        assert!(!is_bf_char(b'\n'));
    }

    #[test]
    fn test_basic_compilation() {
        let program = b"+-><.,";
        let instructions = compile(program).unwrap();
        assert_eq!(instructions.len(), 6);
        assert!(matches!(instructions[0], Instruction::Add(1)));
        assert!(matches!(instructions[1], Instruction::Sub(1)));
        assert!(matches!(instructions[2], Instruction::MoveRight(1)));
        assert!(matches!(instructions[3], Instruction::MoveLeft(1)));
        assert!(matches!(instructions[4], Instruction::Output));
        assert!(matches!(instructions[5], Instruction::Input));
    }

    #[test]
    fn test_instruction_folding() {
        let program = b"++++--->>>><<";
        let instructions = compile(program).unwrap();
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[0], Instruction::Add(4));
        assert_eq!(instructions[1], Instruction::Sub(3));
        assert_eq!(instructions[2], Instruction::MoveRight(4));
        assert_eq!(instructions[3], Instruction::MoveLeft(2));
    }

    #[test]
    fn test_wrapping_arithmetic_folding() {
        let mut program = vec![b'+'; 256];
        let instructions = compile(&program).unwrap();
        assert!(instructions.is_empty());

        program.push(b'+');
        let instructions = compile(&program).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::Add(1));
    }

    #[test]
    fn test_clear_optimization() {
        let program = b"[-]";
        let instructions = compile(program).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], Instruction::Clear);

        let program_plus = b"[+]";
        let instructions_plus = compile(program_plus).unwrap();
        assert_eq!(instructions_plus.len(), 1);
        assert_eq!(instructions_plus[0], Instruction::Clear);

        let program_ignored = b"[ - ]";
        let instructions_ignored = compile(program_ignored).unwrap();
        assert_eq!(instructions_ignored.len(), 1);
        assert_eq!(instructions_ignored[0], Instruction::Clear);
    }

    #[test]
    fn test_syntax_errors() {
        assert_eq!(compile(b"[+").unwrap_err(), CompileError::UnmatchedOpenBracket);
        assert_eq!(compile(b"+]").unwrap_err(), CompileError::UnmatchedCloseBracket);
        assert_eq!(compile(b"[[+]").unwrap_err(), CompileError::UnmatchedOpenBracket);
        assert_eq!(compile(b"[+]]").unwrap_err(), CompileError::UnmatchedCloseBracket);
    }

    #[test]
    fn test_jump_resolution() {
        let program = b"[>+<-]";
        let instructions = compile(program).unwrap();
        assert_eq!(instructions.len(), 6);
        assert_eq!(instructions[0], Instruction::JumpIfZero(6));
        assert_eq!(instructions[5], Instruction::JumpIfNotZero(1));
    }

    #[test]
    fn test_nested_loops_jump_resolution() {
        let program = b"[>[<-]<-]";
        let instructions = compile(program).unwrap();
        assert_eq!(instructions.len(), 9);
        assert_eq!(instructions[0], Instruction::JumpIfZero(9));
        assert_eq!(instructions[2], Instruction::JumpIfZero(6));
        assert_eq!(instructions[5], Instruction::JumpIfNotZero(3));
        assert_eq!(instructions[8], Instruction::JumpIfNotZero(1));
    }

    #[test]
    fn test_execute_basic_add_sub() {
        let instructions = vec![
            Instruction::Add(5),
            Instruction::Sub(2),
        ];
        let mut output = Vec::new();
        execute(&instructions, io::empty(), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_pointer_underflow() {
        let instructions = vec![
            Instruction::MoveLeft(1),
        ];
        let res = execute(&instructions, io::empty(), io::sink());
        assert_eq!(res.unwrap_err(), RuntimeError::PointerUnderflow);
    }

    #[test]
    fn test_pointer_move_and_resize() {
        let instructions = vec![
            Instruction::MoveRight(30005),
            Instruction::Add(42),
            Instruction::Output,
        ];
        let mut output = Vec::new();
        execute(&instructions, io::empty(), &mut output).unwrap();
        assert_eq!(output, vec![42]);
    }

    #[test]
    fn test_input_eof() {
        let instructions = vec![
            Instruction::Input,
            Instruction::Add(5),
            Instruction::Output,
        ];
        let mut output = Vec::new();
        execute(&instructions, io::empty(), &mut output).unwrap();
        assert_eq!(output, vec![5]);
    }

    #[test]
    fn test_input_read() {
        let input = vec![65, 66];
        let instructions = vec![
            Instruction::Input,
            Instruction::Output,
            Instruction::Input,
            Instruction::Output,
        ];
        let mut output = Vec::new();
        execute(&instructions, &input[..], &mut output).unwrap();
        assert_eq!(output, vec![65, 66]);
    }

    #[test]
    fn test_pointer_overflow() {
        let instructions = vec![
            Instruction::MoveRight(usize::MAX),
            Instruction::MoveRight(1),
        ];
        let res = execute(&instructions, io::empty(), io::sink());
        assert_eq!(res.unwrap_err(), RuntimeError::PointerOverflow);
    }

    #[test]
    fn test_invalid_jump_target() {
        let instructions = vec![
            Instruction::JumpIfZero(9999),
        ];
        let res = execute(&instructions, io::empty(), io::sink());
        assert_eq!(res.unwrap_err(), RuntimeError::InvalidJumpTarget);
    }
}
