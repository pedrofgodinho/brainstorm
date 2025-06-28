pub mod debugger;

use crate::parser::{Program, Token};
use clap::ValueEnum;
use owo_colors::{OwoColorize, Style};
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Write;
use std::io::{Read, Write as _};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Tried to move outside of tape")]
    TapeOverrun,
    #[error("Invalid program: tried to jump outside of the program")]
    InvalidProgram,
    #[error("Failed to read input")]
    InputError,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum EofBehaviour {
    SetZero,
    SetMinusOne,
    DontSet,
}

impl Display for EofBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetZero => write!(f, "set-zero"),
            Self::SetMinusOne => write!(f, "set-minus-one"),
            Self::DontSet => write!(f, "dont-set"),
        }
    }
}

pub struct Interpreter<R: Read> {
    tape: Vec<u8>,
    program: Program,
    pc: usize,
    ptr: usize,
    input: R,
    eof_behaviour: EofBehaviour,
    current_unit: usize,
    breakpoints: HashSet<usize>,
}

impl<R: Read> Interpreter<R> {
    /// Create a new brainstorm interpreter
    pub fn new(
        program: Program,
        tape_size: usize,
        eof_behaviour: EofBehaviour,
        input: R,
    ) -> Interpreter<R> {
        Interpreter {
            tape: vec![0; tape_size],
            program,
            pc: 0,
            ptr: 0,
            input,
            eof_behaviour,
            breakpoints: HashSet::new(),
            current_unit: 0,
        }
    }

    fn hexdump_line(&self, start: usize, width: usize) {
        print!(" {:#0width$x}  ", start.yellow());
        for i in 0..16 {
            if i == 8 {
                print!(" ");
            }
            if start + i < self.tape.len() {
                if start + i == self.ptr {
                    print!("{:02X} ", self.tape[start + i].green());
                } else {
                    print!("{:02X} ", self.tape[start + i]);
                };
            } else {
                print!("   ");
            }
        }

        print!("   ");

        for i in 0..16 {
            if start + i >= self.tape.len() {
                break;
            }
            if i == 8 {
                print!(" ");
            }
            let char = self.tape[start + i];
            let char = if (32..=176).contains(&char) {
                char as char
            } else {
                '·'
            };
            if start + i == self.ptr {
                print!("{} ", char.green());
            } else {
                print!("{char} ");
            }
        }

        println!();
    }

    fn dump_program_range(
        &self,
        unit_name: &str,
        start: usize,
        end: usize,
        indentation: &mut usize,
    ) -> (String, Option<usize>) {
        // This entire function is beyond ugly, as I just kept expanding it to add more features rather than refactoring
        // I might fix it at some point.
        let mut output = String::new();
        let width = format!("{:#x}", self.program.tokens.len() - 1).len();
        let mut green_line = None;

        const EMPTY: &str = "";

        // The write! macro might return an error, but writing to a String cannot fail,
        // so .unwrap() is safe to use here.
        write!(
            output,
            "{:#0width$x}  {}",
            start.yellow(),
            unit_name.yellow()
        )
        .unwrap();

        let mut next_on_new_line = true;
        let mut tokens_since_new_line = 0;
        for i in start..end {
            let token = self.program.tokens[i];
            match token {
                // The two match arms were identical, so they can be combined.
                Token::JumpNotZero(_) => {
                    *indentation -= 2;
                    next_on_new_line = true;
                }
                Token::JumpZero(_) => {
                    next_on_new_line = true;
                }
                _ => (),
            }

            if next_on_new_line || tokens_since_new_line >= 5 {
                write!(
                    output,
                    "\n{:#0width$x}    {EMPTY: <indentation$}",
                    i.dimmed()
                )
                .unwrap();
                next_on_new_line = false;
                tokens_since_new_line = 0;
            }

            let mut style = Style::new();
            if self.breakpoints.contains(&i) {
                style = style.underline().red();
            }
            if i == self.pc {
                style = style.green();
                green_line = Some(output.lines().count() - 1);
            }
            write!(output, "{} ", token.style(style)).unwrap();

            match token {
                Token::JumpNotZero(t) => {
                    next_on_new_line = true;
                    write!(output, " {} {:#x}", "->".dimmed(), (t - 1).dimmed()).unwrap();
                }
                Token::JumpZero(t) => {
                    *indentation += 2;
                    next_on_new_line = true;
                    write!(output, " {} {:#x}", "->".dimmed(), (t - 1).dimmed()).unwrap();
                }
                _ => (),
            }

            tokens_since_new_line += 1;
        }

        // writeln! appends a newline character at the end.
        writeln!(output).unwrap();

        (output, green_line)
    }

    /// Dumps the entire program to a string, and an usize indicating the line that includes the
    /// current instruction
    pub fn dump_program(&self) -> (String, usize) {
        let mut result = String::new();
        let mut green_line = 0;
        let mut line_count = 0;
        let mut indentation = 0;
        for unit in &self.program.units {
            let unit_dump =
                self.dump_program_range(&unit.description, unit.start, unit.end, &mut indentation);
            result.push_str(&unit_dump.0);
            if let Some(line) = unit_dump.1 {
                green_line = line_count + line;
            }
            line_count += unit_dump.0.lines().count();
        }
        (result, green_line)
    }

    /// Dumps a certain amount of lines before and after the current executing instruction
    pub fn dump_current_program_section(&self, before: usize, after: usize) {
        let dump = self.dump_program();
        println!("Printing {} to {} around {}", before, after, dump.1);
        for line in dump
            .0
            .lines()
            .skip(dump.1.saturating_sub(before))
            .take(before + 1 + after)
        {
            println!("{line}");
        }
    }

    /// Prints a hexdump of the tape, skipping over lines that are at zero
    pub fn print_tape(&self) {
        let address_width = format!("{:#x}", self.tape.len()).len();

        let mut first_all_zeroes = false;
        let mut ellipsis = false;

        for i in (0..self.tape.len()).step_by(16) {
            if self.tape[i..i + 16].iter().all(|&c| c == 0) {
                if !first_all_zeroes {
                    self.hexdump_line(i, address_width);
                    first_all_zeroes = true;
                } else if !ellipsis {
                    println!("{: <width$}   ....", "", width = address_width);
                    ellipsis = true;
                }
                continue;
            } else {
                first_all_zeroes = false;
                ellipsis = false;
            }
            self.hexdump_line(i, address_width);
        }
    }

    /// Prints the internal state of the interpreter
    pub fn print_state(&self) {
        println!(
            "{}", "============================================= CTX =============================================".red()
        );

        println!("{}", "Tape:".blue().bold());
        self.print_tape();
        println!();

        println!("{}", "Program:".blue().bold());
        self.dump_current_program_section(5, 5);

        println!();
        println!("{}", "Registers:".blue().bold());
        println!("{}: {:#0x}", "PC".yellow(), self.pc);
        println!("{}: {:#0x}", "TP".yellow(), self.ptr);
        println!(
            "{}: {}",
            "Current Unit".yellow(),
            self.program.units[self.current_unit].description
        );

        println!(
            "{}", "=========================================== END CTX ===========================================".red()
        );
    }

    /// Takes a single step in the interpreter. Returns OK(true) if there's still more program to
    /// execute, and Ok(false) if the program has halted (reached EOF). May return an error if the
    /// brainfuck program tries to move outside the tape, or if IO fails
    pub fn step(&mut self) -> Result<bool, InterpreterError> {
        match self
            .program
            .tokens
            .get(self.pc)
            .ok_or(InterpreterError::InvalidProgram)?
        {
            Token::Increment(value) => {
                self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(*value)
            }
            Token::Move(value) => {
                if self.ptr.wrapping_add(*value as usize) >= self.tape.len() {
                    return Err(InterpreterError::TapeOverrun);
                }
                self.ptr = self.ptr.wrapping_add(*value as usize);
            }
            Token::JumpZero(value) => {
                if self.tape[self.ptr] == 0 {
                    self.pc = *value - 1
                }
            }
            Token::JumpNotZero(value) => {
                if self.tape[self.ptr] != 0 {
                    self.pc = *value - 1
                }
            }
            Token::Output => {
                print!("{}", self.tape[self.ptr] as char);
                std::io::stdout().flush().unwrap();
            }
            Token::Input => {
                let mut buffer = [0u8; 1];
                let mut bytes = self.input.read(&mut buffer);
                if buffer[0] == b'\r' {
                    bytes = self.input.read(&mut buffer); // skip carriage return
                }
                match bytes {
                    Ok(0) => match self.eof_behaviour {
                        EofBehaviour::SetZero => self.tape[self.ptr] = 0,
                        EofBehaviour::SetMinusOne => self.tape[self.ptr] = 255,
                        EofBehaviour::DontSet => (),
                    },
                    Ok(_) => self.tape[self.ptr] = buffer[0],
                    Err(_) => return Err(InterpreterError::InputError),
                }
            }
            Token::PrintState => self.print_state(),
            Token::Eof => return Ok(false),
        }
        self.pc += 1;

        while !(self.program.units[self.current_unit].start
            ..self.program.units[self.current_unit].end)
            .contains(&self.pc)
        {
            self.current_unit += 1;
            self.current_unit %= self.program.units.len();
        }

        Ok(true)
    }

    /// Steps instructions until the current unit is left. Returns OK(true) if there's still more program to
    /// execute, and Ok(false) if the program has halted (reached EOF). May return an error if the
    /// brainfuck program tries to move outside the tape, or if IO fails
    pub fn step_unit(&mut self) -> Result<bool, InterpreterError> {
        let starting_unit = self.current_unit;
        while self.step()? {
            if self.current_unit != starting_unit {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Runs the program until it halts (reached EOF).
    pub fn run(&mut self) -> Result<(), InterpreterError> {
        while self.step()? {}
        Ok(())
    }

    /// Adds a breakpoint. Breakpoints are only considered in the `Interpreter::cont` function
    pub fn add_breakpoint(&mut self, breakpoint: usize) {
        self.breakpoints.insert(breakpoint);
    }

    /// Clears a breakpoint. Returns true if successful, returns false if no such breakpoint existed
    pub fn clear_breakpoint(&mut self, breakpoint: usize) -> bool {
        self.breakpoints.remove(&breakpoint)
    }

    /// Runs the program until it halts (reached EOF) or until it hits a breakpoint.
    pub fn cont(&mut self) -> Result<bool, InterpreterError> {
        while self.step()? {
            if self.breakpoints.contains(&self.pc) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
