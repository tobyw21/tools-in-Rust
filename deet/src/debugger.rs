use std::collections::HashMap;

use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::Inferior;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::inferior::Status;
use crate::disassembler::DisassembleObject;

fn parse_address(addr: &str) -> Option<usize> {
    let addr_without0x = if addr.to_lowercase().starts_with("*0x") {
        &addr[3..]
    } else {
        &addr
    };

    usize::from_str_radix(addr_without0x, 16).ok()
}

#[derive(Clone)]
pub struct BreakPoint {
    pub addr: usize,
    pub orig_byte: u8,
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: Vec<usize>,
    breakpoint_set: HashMap<usize, BreakPoint>,
    disassemble: DisassembleObject,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                eprintln!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                eprintln!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);
        let breakpoints = Vec::new();
        let breakpoint_set: HashMap<usize, BreakPoint> = HashMap::new();

        // create disassemble object
        let disassemble = DisassembleObject::new(target);
        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints,
            breakpoint_set,
            disassemble,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    self.to_kill();

                    if let Some(inferior) = Inferior::new(
                        &self.target,
                        &args,
                        &self.breakpoints,
                        &mut self.breakpoint_set,
                    ) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        let inferior_object = self.inferior.as_mut().unwrap();
                        let status = inferior_object.cont_exec(&self.breakpoint_set);
                        match status {
                            Ok(stat) => {
                                println!("Child {}", stat);

                                match stat {
                                    Status::Stopped(_sig, _rip) => {
                                        let ref mut debug_data_ref = self.debug_data;
                                        let line = self
                                            .inferior
                                            .as_mut()
                                            .unwrap()
                                            .get_stop_line(debug_data_ref);

                                        if line.is_some() {
                                            println!("Stopped at {}", line.unwrap());
                                        }
                                    }
                                    _ => (),
                                }
                            }

                            Err(e) => eprintln!("{}", e),
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }

                DebuggerCommand::Cont => {
                    if self.inferior.is_none() {
                        eprintln!("No child process is running!");
                        continue;
                    }
                    let result = self
                        .inferior
                        .as_mut()
                        .unwrap()
                        .cont_exec(&self.breakpoint_set);

                    match result {
                        Ok(stat) => println!("Child {}", stat),
                        Err(e) => eprintln!("{}", e),
                    }
                }

                DebuggerCommand::Back => {
                    if self.inferior.is_none() {
                        eprintln!("No child process is running!");
                        continue;
                    }

                    let ref mut debug_data_ref = self.debug_data;

                    let result = self
                        .inferior
                        .as_mut()
                        .unwrap()
                        .print_backtrace(debug_data_ref);
                    match result {
                        Ok(()) => (),
                        Err(e) => eprintln!("{}", e),
                    }
                }

                DebuggerCommand::Break(arg) => {
                    let breakpoint: Option<usize>;
                    if arg.starts_with("*") {
                        if let Some(parsed_addr) = parse_address(&arg) {
                            breakpoint = Some(parsed_addr);
                        } else {
                            breakpoint = None;
                        }
                    } else if let Some(line) = usize::from_str_radix(&arg, 10).ok() {
                        if let Some(line_addr) = self.debug_data.get_addr_for_line(None, line) {
                            breakpoint = Some(line_addr);
                        } else {
                            breakpoint = None;
                        }
                    } else if let Some(func_addr) =
                        self.debug_data.get_addr_for_function(None, &arg)
                    {
                        breakpoint = Some(func_addr);
                    } else {
                        breakpoint = None;
                    }

                    match breakpoint {
                        Some(bp) => {
                            if !self.breakpoints.contains(&bp) {
                                if self.inferior.is_some() {
                                    if let Some(orig_byte) =
                                        self.inferior.as_mut().unwrap().write_byte(bp, 0xcc).ok()
                                    {
                                        self.breakpoints.push(bp);

                                        let nbreakpoints = self.breakpoints.len() - 1;
                                        println!("Set breakpoint {} at {:#x}", nbreakpoints, bp);

                                        self.breakpoint_set.insert(
                                            bp,
                                            BreakPoint {
                                                addr: bp,
                                                orig_byte,
                                            },
                                        );
                                    }
                                } else {
                                    self.breakpoints.push(bp);

                                    let nbreakpoints = self.breakpoints.len() - 1;
                                    println!("Set breakpoint {} at {:#x}", nbreakpoints, bp);
                                }
                            } else {
                                println!("Breakpoint {:#x} exists", bp);
                            }
                        }

                        None => println!("Function {} not defined!", arg),
                    }
                }

                DebuggerCommand::Info(arg) => match arg.as_str() {
                    "b" | "breakpoints" => {
                        for (index, breakpoint) in self.breakpoints.iter().enumerate() {
                            println!("#{} = {:#x}", index, breakpoint);
                        }
                    }
                    _ => (),
                },

                #[allow(unused_variables)]
                DebuggerCommand::Disassemble() => {
                    // arg is where user want it to be disassembled
                    // not yet implmented
                    self.disassemble.disassemble();
                }

                DebuggerCommand::Quit => {
                    self.to_kill();
                    return;
                }
            }
        }
    }

    /// kill the current running inferior
    fn to_kill(&mut self) {
        if !self.inferior.is_none() {
            self.breakpoints.clear();
            let pid = self.inferior.as_mut().unwrap().pid();
            self.inferior.as_mut().unwrap().kill_inferior();
            println!("Killing running inferior (pid {})", pid);
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
