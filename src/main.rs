use crate::interpreter::debugger::Debugger;
use crate::interpreter::{EofBehaviour, Interpreter};
use crate::parser::Program;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod interpreter;
mod parser;

/// A brainfuck interpreter and debugger
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Sets the program file to run
    #[arg(short, long)]
    program_file: PathBuf,

    /// Sets the size of the tape for the interpreter
    #[arg(short, long, default_value_t = 1024*64)]
    tape_size: usize,

    /// Sets the behaviour when an input instruction is executed after input has reached end of file
    #[arg(short, long, default_value_t = EofBehaviour::DontSet)]
    eof_behaviour: EofBehaviour,

    /// Enables printing the interpreter's internal status on # commands
    #[arg(short = 'i', long, default_value_t = false)]
    print_debug: bool,

    /// Enables the interactive debugger
    #[arg(short, long, default_value_t = false)]
    debugger: bool,
}

fn main() {
    let args = Args::parse();

    let file = match File::open(args.program_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening program file: {e}");
            return;
        }
    };

    let reader = BufReader::new(file);
    let program = match Program::parse(reader, args.print_debug) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error parsing program: {e}");
            return;
        }
    };

    let mut interpreter = Interpreter::new(
        program,
        args.tape_size,
        args.eof_behaviour,
        BufReader::new(std::io::stdin()),
    );

    if args.debugger {
        let mut debugger = Debugger::new(interpreter);
        debugger.run();
    } else {
        match interpreter.run() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error running interpreter: {e}");
            }
        }
    }
}
