use nix::sys::ptrace;
use nix::sys::ptrace::getregs;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::fmt;


use crate::dwarf_data::DwarfData;

#[derive(Debug)]
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

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>) -> Option<Inferior> {
        
        let mut command = Command::new(target);
        unsafe {
            command
            .args(args)
            .pre_exec(|| {
                child_traceme()
            });     
            
        };
        
        if let Ok(child_process) = command.spawn() {
            let child_pid = nix::unistd::Pid::from_raw(child_process.id() as i32);
            let pid_result = waitpid(child_pid, 
                        Some(WaitPidFlag::WSTOPPED));
            
            match pid_result {
                Ok(status) => 
                    if status.eq(&WaitStatus::Stopped(child_pid, signal::SIGTRAP)) {
                        Some(
                            Inferior {
                                child: child_process,
                            }
                        )

                    } else {
                        None
                    },
                Err(_e) => None,
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
    pub fn cont_exec(&self) -> Result<Status, nix::Error> {
        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    /// kill the inferior and reap it
    pub fn kill_inferior(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }

    pub fn print_backtrace(&self, debug_data: &mut DwarfData) -> Result<(), nix::Error> {

        let register_value = getregs(self.pid()).ok().unwrap().rip as usize;
        let line = debug_data.get_line_from_addr(register_value).unwrap();
        let func_name = debug_data.get_function_from_addr(register_value).unwrap();
        Ok(println!("{} ({})", func_name, line))

    }
    
}
