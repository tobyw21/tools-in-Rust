use nix::sys::ptrace::getregs;
use nix::sys::ptrace::{self, setregs};
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::fmt;
use std::mem::size_of;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::process::Command;

use crate::debugger::BreakPoint;

use crate::dwarf_data::{self, DwarfData};

#[derive(PartialEq)]
pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Exited(code) => write!(f, "exited (status {})", code),
            Self::Stopped(sig, _eip) => write!(f, "stopped (signal {})", sig),
            Self::Signaled(sig) => write!(f, "{}", sig),
        }
    }
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(
        target: &str,
        args: &Vec<String>,
        breakset: &Vec<usize>,
        breakpoint_set: &mut HashMap<usize, BreakPoint>,
    ) -> Option<Inferior> {
        let mut command = Command::new(target);
        unsafe {
            command.args(args).pre_exec(|| child_traceme());
        };

        if let Ok(child_process) = command.spawn() {
            let child_pid = nix::unistd::Pid::from_raw(child_process.id() as i32);

            // wait for SIGTRAP
            let pid_result = waitpid(child_pid, Some(WaitPidFlag::WSTOPPED));

            match pid_result {
                Ok(status) => {
                    if status.eq(&WaitStatus::Stopped(child_pid, signal::SIGTRAP)) {
                        // write to the process
                        // given address
                        // use ptrace::write
                        // make 0xcc, int, interupt to the address
                        // in order to make a 'break point'
                        let mut final_inferior = Inferior {
                            child: child_process,
                        };
                        for breakpoint in breakset {
                            let ret_byte = final_inferior
                                .write_byte(*breakpoint, 0xcc)
                                .expect("unable to write to break point");

                            breakpoint_set.insert(
                                *breakpoint,
                                BreakPoint {
                                    addr: *breakpoint,
                                    orig_byte: ret_byte,
                                },
                            );
                        }

                        // return the new inferior created
                        Some(final_inferior)
                    } else {
                        None
                    }
                }
                Err(e) => panic!("Unexpected error {:?} on spawning process", e),
            }
        } else {
            None
        }
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    /// continue execute trapped process
    pub fn cont_exec(
        &mut self,
        breakpoint_set: &HashMap<usize, BreakPoint>,
    ) -> Result<Status, nix::Error> {
        let mut regs = getregs(self.pid())?;
        let rip_val = regs.rip as usize;

        if let Some(bpoint) = breakpoint_set.get(&(rip_val - 1)) {
            self.write_byte(rip_val - 1, bpoint.orig_byte).ok();
            regs.rip = (rip_val - 1) as u64;

            setregs(self.pid(), regs).ok();

            ptrace::step(self.pid(), None).ok();

            match self.wait(None).ok().unwrap() {
                Status::Exited(code) => return Ok(Status::Exited(code)),
                Status::Signaled(sig) => return Ok(Status::Signaled(sig)),
                Status::Stopped(_sig, _reg) => {
                    self.write_byte(rip_val - 1, 0xcc).ok();
                }
            }
        }

        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    /// kill the inferior and reap it
    pub fn kill_inferior(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }

    /// print the backtrace of current debugging process
    pub fn print_backtrace(&self, debug_data: &mut DwarfData) -> Result<(), nix::Error> {
        let mut rip_value = getregs(self.pid()).ok().unwrap().rip as usize;
        let mut rbp_value = getregs(self.pid()).ok().unwrap().rbp as usize;
        let mut line = debug_data.get_line_from_addr(rip_value).unwrap();
        let mut func_name = debug_data.get_function_from_addr(rip_value).unwrap();

        loop {
            println!("{} ({})", func_name, line);
            if func_name.contains("main") {
                break;
            }

            rip_value = ptrace::read(self.pid(), (rbp_value + 8) as ptrace::AddressType)? as usize;
            rbp_value = ptrace::read(self.pid(), rbp_value as ptrace::AddressType)? as usize;
            line = debug_data.get_line_from_addr(rip_value).unwrap();
            func_name = debug_data.get_function_from_addr(rip_value).unwrap();
        }

        Ok(())
    }

    pub fn get_stop_line(&self, debug_data: &mut DwarfData) -> Option<dwarf_data::Line> {
        let rip_value = getregs(self.pid()).ok().unwrap().rip as usize;
        let line = debug_data.get_line_from_addr(rip_value);

        line
    }

    pub fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);

        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;

        Ok(orig_byte as u8)
    }
}
