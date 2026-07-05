use std::fs;
use std::io;
use brainfuck_rs::{compile, execute};

#[test]
fn test_integration_hello_world_file() {
    let program = fs::read("examples/helloworld.b").expect("Failed to read helloworld.b");
    let instructions = compile(&program).expect("Failed to compile helloworld.b");
    
    let mut output = Vec::new();
    execute(&instructions, io::empty(), &mut output).expect("Failed to execute helloworld.b");
    
    let output_str = String::from_utf8(output).expect("Output is not valid UTF-8");
    assert_eq!(output_str, "Hello world!\n");
}

#[test]
fn test_integration_echo() {
    // Echo program: ,[.,]
    let program = b",[.,]";
    let instructions = compile(program).expect("Failed to compile echo program");
    
    let input = b"Hello, Brainfuck!";
    let mut output = Vec::new();
    execute(&instructions, &input[..], &mut output).expect("Failed to execute echo program");
    
    assert_eq!(output, input);
}

#[test]
fn test_integration_add_digits() {
    // Simple addition of two inputs: ,>,<[->+<]>.
    let program = b",>,<[->+<]>.";
    let instructions = compile(program).expect("Failed to compile add program");
    
    let input = vec![5, 10];
    let mut output = Vec::new();
    execute(&instructions, &input[..], &mut output).expect("Failed to execute add program");
    
    assert_eq!(output, vec![15]);
}

#[test]
fn test_integration_multiply_digits() {
    // Multiplies cell 0 by cell 1, puts result in cell 2.
    // Formula: ,>, < [ > [ >+>+<<- ] >> [ <<+>>- ] <<<- ] >>.
    let program = b",>,<[>[>+>+<<-]>>[<<+>>-]<<<-]>>.";
    let instructions = compile(program).expect("Failed to compile multiply program");
    
    // 6 * 7 = 42 (ascii '*', which is 42)
    let input = vec![6, 7];
    let mut output = Vec::new();
    execute(&instructions, &input[..], &mut output).expect("Failed to execute multiply program");
    
    assert_eq!(output, vec![42]);
}

#[test]
fn test_integration_pointer_wrapping_and_underflow() {
    // Cell moves left and triggers underflow error
    let program = b"<";
    let instructions = compile(program).unwrap();
    let res = execute(&instructions, io::empty(), io::sink());
    assert!(res.is_err());
}

#[test]
fn test_integration_clear_optimization_behavior() {
    // Sets cell to 5, then clears it. Output should be 0.
    let program = b"+++++[-].";
    let instructions = compile(program).unwrap();
    
    let mut output = Vec::new();
    execute(&instructions, io::empty(), &mut output).unwrap();
    assert_eq!(output, vec![0]);
}

#[test]
fn test_integration_nested_loops_and_conditionals() {
    // Set cell 0 = 3, cell 1 = 2
    // While cell 0 is not zero:
    //   While cell 1 is not zero:
    //     Increment cell 2
    //     Decrement cell 1
    //   Decrement cell 0
    // Result in cell 2 should be 2.
    let program = b"+++>++<[>[>+<-]<-]>>.";
    let instructions = compile(program).unwrap();
    
    let mut output = Vec::new();
    execute(&instructions, io::empty(), &mut output).unwrap();
    assert_eq!(output, vec![2]);
}

#[test]
fn test_integration_alphabet_generator() {
    // Generates "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    let program = b"+++++ [ >+++++ <- ] >+ [- < + >] ++++++ [ >++++++++++ <- ] >+++++ << [ >> . + << - ]";
    let instructions = compile(program).unwrap();
    
    let mut output = Vec::new();
    execute(&instructions, io::empty(), &mut output).unwrap();
    
    let output_str = String::from_utf8(output).unwrap();
    assert_eq!(output_str, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
}

#[test]
fn test_integration_fibonacci() {
    // Generates first 6 Fibonacci numbers and outputs them as raw bytes: 1, 1, 2, 3, 5, 8
    let program = b"+.>+.>>>++++[<<<[>+>+<<-] <[>>+<<-] >>>[<<<+>>>-] <[<+>-] <.>>>-]";
    let instructions = compile(program).unwrap();
    
    let mut output = Vec::new();
    execute(&instructions, io::empty(), &mut output).unwrap();
    assert_eq!(output, vec![1, 1, 2, 3, 5, 8]);
}
