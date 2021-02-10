use crate::errno::Errno;
use crate::sys::signal::Signal;
use crate::unistd::Pid;
use crate::Result;
use cfg_if::cfg_if;
use libc::{self, c_int};
use std::convert::{From, Into, TryFrom};

libc_bitflags!(
    pub struct WaitPidFlag: c_int {
        WNOHANG;
        WUNTRACED;
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        WEXITED;
        WCONTINUED;
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        WSTOPPED;
        /// Don't reap, just poll status.
        #[cfg(any(target_os = "android",
                  target_os = "freebsd",
                  target_os = "haiku",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "redox",
                  target_os = "macos",
                  target_os = "netbsd"))]
        WNOWAIT;
        /// Don't wait on children of other threads in this group
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        __WNOTHREAD;
        /// Wait on all children, regardless of type
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        __WALL;
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        __WCLONE;
    }
);

/// Possible child targets of `waitpid()`.
///
/// The most common variants for waiting are waiting for a specific
/// child via `WaitPid::Child(pid)` and waiting for any child via
/// `WaitPid::AnyChild`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WaitTarget {
    /// A child process with `Pid` process id.
    Child(Pid),
    /// Any child process of the same process group as the calling process.
    SameGroupChild,
    /// Any child process.
    AnyChild,
    /// Any child process of `Pid` process group id. Note, do not negate
    /// this value, use process id of group leader instead.
    ChildOfGroup(Pid),
}

impl From<Pid> for WaitTarget {
    /// Convert libc `pid_t` to `waitpid`'s `pid` as Rust safe `WaitTarget`.
    fn from(pid: Pid) -> WaitTarget {
        match pid.as_raw() {
            i if i > 0 => WaitTarget::Child(pid),
            -1 => WaitTarget::AnyChild,
            0 => WaitTarget::SameGroupChild,
            _ => WaitTarget::ChildOfGroup(pid),
        }
    }
}

/// Possible return values from `wait()` or `waitpid()`.
///
/// Each status (other than `StillAlive`) describes a state transition
/// in a child process `Pid`, such as the process exiting or stopping,
/// plus additional data about the transition if any.
///
/// Note that there are two Linux-specific enum variants, `PtraceEvent`
/// and `PtraceSyscall`. Portable code should avoid exhaustively
/// matching on `WaitStatus`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WaitStatus {
    /// The process exited normally (as with `exit()` or returning from
    /// `main`) with the given exit code. This case matches the C macro
    /// `WIFEXITED(status)`; the second field is `WEXITSTATUS(status)`.
    Exited(Pid, i32),
    /// The process was killed by the given signal. The third field
    /// indicates whether the signal generated a core dump. This case
    /// matches the C macro `WIFSIGNALED(status)`; the last two fields
    /// correspond to `WTERMSIG(status)` and `WCOREDUMP(status)`.
    Signaled(Pid, Signal, bool),
    /// The process is alive, but was stopped by the given signal. This
    /// is only reported if `WaitPidFlag::WUNTRACED` was passed. This
    /// case matches the C macro `WIFSTOPPED(status)`; the second field
    /// is `WSTOPSIG(status)`.
    Stopped(Pid, Signal),
    /// The traced process was stopped by a `PTRACE_EVENT_*` event. See
    /// [`nix::sys::ptrace`] and [`ptrace`(2)] for more information. All
    /// currently-defined events use `SIGTRAP` as the signal; the third
    /// field is the `PTRACE_EVENT_*` value of the event.
    ///
    /// [`nix::sys::ptrace`]: ../ptrace/index.html
    /// [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PtraceEvent(Pid, Signal, c_int),
    /// The traced process was stopped by execution of a system call,
    /// and `PTRACE_O_TRACESYSGOOD` is in effect. See [`ptrace`(2)] for
    /// more information.
    ///
    /// [`ptrace`(2)]: http://man7.org/linux/man-pages/man2/ptrace.2.html
    #[cfg(any(target_os = "linux", target_os = "android"))]
    PtraceSyscall(Pid),
    /// The process was previously stopped but has resumed execution
    /// after receiving a `SIGCONT` signal. This is only reported if
    /// `WaitPidFlag::WCONTINUED` was passed. This case matches the C
    /// macro `WIFCONTINUED(status)`.
    Continued(Pid),
    /// There are currently no state changes to report in any awaited
    /// child process. This is only returned if `WaitPidFlag::WNOHANG`
    /// was used (otherwise `wait()` or `waitpid()` would block until
    /// there was something to report).
    StillAlive,
}

impl WaitStatus {
    /// Extracts the PID from the WaitStatus unless it equals StillAlive.
    pub fn pid(&self) -> Option<Pid> {
        use self::WaitStatus::*;
        match *self {
            Exited(p, _)  | Signaled(p, _, _) |
                Stopped(p, _) | Continued(p) => Some(p),
            StillAlive => None,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            PtraceEvent(p, _, _) | PtraceSyscall(p) => Some(p),
        }
    }
}

#[allow(unused_unsafe)]
fn exited(status: i32) -> bool {
    unsafe { libc::WIFEXITED(status) }
}

#[allow(unused_unsafe)]
fn exit_status(status: i32) -> i32 {
    unsafe { libc::WEXITSTATUS(status) }
}

#[allow(unused_unsafe)]
fn signaled(status: i32) -> bool {
    unsafe { libc::WIFSIGNALED(status) }
}

#[allow(unused_unsafe)]
fn term_signal(status: i32) -> Result<Signal> {
    Signal::try_from(unsafe { libc::WTERMSIG(status) })
}

#[allow(unused_unsafe)]
fn dumped_core(status: i32) -> bool {
    unsafe { libc::WCOREDUMP(status) }
}

#[allow(unused_unsafe)]
fn stopped(status: i32) -> bool {
    unsafe { libc::WIFSTOPPED(status) }
}

#[allow(unused_unsafe)]
fn stop_signal(status: i32) -> Result<Signal> {
    Signal::try_from(unsafe { libc::WSTOPSIG(status) })
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(unused_unsafe)]
fn syscall_stop(status: i32) -> bool {
    // From ptrace(2), setting PTRACE_O_TRACESYSGOOD has the effect
    // of delivering SIGTRAP | 0x80 as the signal number for syscall
    // stops. This allows easily distinguishing syscall stops from
    // genuine SIGTRAP signals.
    unsafe { libc::WSTOPSIG(status) == libc::SIGTRAP | 0x80 }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn stop_additional(status: i32) -> c_int {
    (status >> 16) as c_int
}

#[allow(unused_unsafe)]
fn continued(status: i32) -> bool {
    unsafe { libc::WIFCONTINUED(status) }
}

impl WaitStatus {
    /// Convert a raw `wstatus` as returned by `waitpid`/`wait` into a `WaitStatus`
    ///
    /// # Errors
    ///
    /// Returns an `Error` corresponding to `EINVAL` for invalid status values.
    ///
    /// # Examples
    ///
    /// Convert a `wstatus` obtained from `libc::waitpid` into a `WaitStatus`:
    ///
    /// ```
    /// use nix::sys::wait::WaitStatus;
    /// use nix::sys::signal::Signal;
    /// let pid = nix::unistd::Pid::from_raw(1);
    /// let status = WaitStatus::from_raw(pid, 0x0002);
    /// assert_eq!(status, Ok(WaitStatus::Signaled(pid, Signal::SIGINT, false)));
    /// ```
    pub fn from_raw(pid: Pid, status: i32) -> Result<WaitStatus> {
        Ok(if exited(status) {
            WaitStatus::Exited(pid, exit_status(status))
        } else if signaled(status) {
            WaitStatus::Signaled(pid, term_signal(status)?, dumped_core(status))
        } else if stopped(status) {
            cfg_if! {
                if #[cfg(any(target_os = "android", target_os = "linux"))] {
                    fn decode_stopped(pid: Pid, status: i32) -> Result<WaitStatus> {
                        let status_additional = stop_additional(status);
                        Ok(if syscall_stop(status) {
                            WaitStatus::PtraceSyscall(pid)
                        } else if status_additional == 0 {
                            WaitStatus::Stopped(pid, stop_signal(status)?)
                        } else {
                            WaitStatus::PtraceEvent(pid, stop_signal(status)?,
                                                    stop_additional(status))
                        })
                    }
                } else {
                    fn decode_stopped(pid: Pid, status: i32) -> Result<WaitStatus> {
                        Ok(WaitStatus::Stopped(pid, stop_signal(status)?))
                    }
                }
            }
            return decode_stopped(pid, status);
        } else {
            assert!(continued(status));
            WaitStatus::Continued(pid)
        })
    }
}

/// Wrapper around `libc::waitpid` that uses [`WaitTarget`] to determine
/// `Pid` target. It waits for the specified `who` target optionally with
/// [`WaitPidFlag`] to specify options for waiting and return the result of
/// [`WaitStatus`].
pub fn waitpid<P: Into<WaitTarget>>(who: P, options: Option<WaitPidFlag>) -> Result<WaitStatus> {
    use self::WaitStatus::*;

    let mut status: i32 = 0;

    let option_bits = match options {
        Some(bits) => bits.bits(),
        None => 0,
    };

    let pid = match who.into() {
        WaitTarget::Child(pid) => pid.into(),
        WaitTarget::SameGroupChild => 0,
        WaitTarget::AnyChild => -1,
        WaitTarget::ChildOfGroup(pid) => {
            let pid: libc::pid_t = pid.into();
            -pid
        }
    };

    let res = unsafe { libc::waitpid(pid, &mut status as *mut c_int, option_bits) };

    match Errno::result(res)? {
        0 => Ok(StillAlive),
        res => WaitStatus::from_raw(Pid::from_raw(res), status),
    }
}

/// Wrapper around `libc::wait` to wait until any child process terminates.
pub fn wait() -> Result<WaitStatus> {
    waitpid(WaitTarget::AnyChild, None)
}
