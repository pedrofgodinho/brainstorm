use crate::interpreter::Interpreter;
use owo_colors::OwoColorize;
use std::io;
use std::io::{Read, Write};

pub struct Debugger<T: Read> {
    interpreter: Interpreter<T>,
    running: bool,
}

impl<T: Read> Debugger<T> {
    pub fn new(interpreter: Interpreter<T>) -> Debugger<T> {
        Debugger {
            interpreter,
            running: true,
        }
    }

    pub fn run(&mut self) {
        let mut last_command;
        let mut input = String::new();

        println!("Welcome to the Brainstorm debugger");
        println!("Use command `help` for information on available commands");

        self.context();

        loop {
            print!("{}", "> ".red());
            io::stdout().flush().unwrap(); // TODO handle this unwrap

            last_command = input.clone();
            input.clear();
            io::stdin().read_line(&mut input).unwrap(); // TODO handle this unwrap

            input = input.trim().to_lowercase();
            if input.is_empty() {
                input = last_command.clone();
            }

            let l = input.trim().to_lowercase();
            if match l.split_whitespace().next().unwrap() {
                "h" | "help" => self.help(),
                "q" | "quit" => {
                    println!("Exiting debugger!");
                    return;
                }
                "ctx" | "context" => self.context(),
                "p" | "program" => self.program(),
                "t" | "tape" => self.tape(),
                "n" | "next" => self.n(),
                "ni" | "next-instruction" => self.ni(),
                "b" | "break" => self.breakpoint(&l),
                "cl" | "clear" => self.clear(&l),
                "c" | "continue" => self.cont(),
                _ => {
                    println!("Unknown command: {l}");
                    false
                }
            } {
                self.context();
            }
        }
    }

    fn help(&self) -> bool {
        println!("Available commands: ");
        println!("  - h / help - prints this message");
        println!("  - q / quit - quits the debugger");
        println!("  - ctx / context - prints the context window");
        println!("  - p / program - prints the entire program units");
        println!("  - t / tape - prints the tape");
        println!("  - n / next - steps the interpreter by one unit");
        println!("  - ni / next-instruction - steps the interpreter by one bf instruction");
        println!("  - b / break - set a breakpoint at the specified location (hex)");
        println!("  - cl / clear - clear a breakpoint at the specified location (hex)");
        println!("  - c / continue - continue execution until breakpoint or halt");
        false
    }

    fn context(&self) -> bool {
        println!();
        self.interpreter.print_state();
        false
    }

    fn program(&self) -> bool {
        println!("{}", self.interpreter.dump_program().0);
        false
    }

    fn tape(&self) -> bool {
        self.interpreter.print_tape();
        false
    }

    fn n(&mut self) -> bool {
        if !self.running {
            println!("Program is halted");
            return false;
        }
        match self.interpreter.step_unit() {
            Ok(true) => (),
            Ok(false) => {
                self.running = false;
                println!("Program has halted");
            }
            Err(e) => {
                self.running = false;
                println!("Program has halted with an error:");
                println!("{e}");
            }
        }
        true
    }

    fn ni(&mut self) -> bool {
        if !self.running {
            println!("Program is halted");
            return false;
        }
        match self.interpreter.step() {
            Ok(true) => (),
            Ok(false) => {
                self.running = false;
                println!("Program has halted");
            }
            Err(e) => {
                self.running = false;
                println!("Program has halted with an error:");
                println!("{e}");
            }
        }
        true
    }

    fn breakpoint(&mut self, l: &str) -> bool {
        if let Some(s) = l.split_whitespace().nth(1)
            && let Ok(v) = usize::from_str_radix(s.trim_start_matches("0x"), 16)
        {
            println!("Added breakpoint at {v:#x}");
            self.interpreter.add_breakpoint(v);
        } else {
            println!("Invalid breakpoint");
        }
        false
    }

    fn clear(&mut self, l: &str) -> bool {
        if let Some(s) = l.split_whitespace().nth(1)
            && let Ok(v) = usize::from_str_radix(s.trim_start_matches("0x"), 16)
        {
            if self.interpreter.clear_breakpoint(v) {
                println!("Cleared breakpoint at {v:#x}");
            } else {
                println!("No breakpoint at {v:#x}");
            }
        } else {
            println!("Invalid breakpoint");
        }
        false
    }

    fn cont(&mut self) -> bool {
        if !self.running {
            println!("Program is halted");
            return false;
        }
        match self.interpreter.cont() {
            Ok(true) => (),
            Ok(false) => {
                self.running = false;
                println!("Program has halted");
            }
            Err(e) => {
                self.running = false;
                println!("Program has halted with an error:");
                println!("{e}");
            }
        }
        true
    }
}
