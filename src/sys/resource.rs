//! Configure the process resource limits.
use cfg_if::cfg_if;

use crate::errno::Errno;
use crate::Result;

pub use libc::rlim_t;

cfg_if! {
    if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
        use libc::{__rlimit_resource_t, rlimit, RLIM_INFINITY};
        libc_enum!{
            #[repr(u32)]
            pub enum Resource {
                /// See detail of each Resource https://man7.org/linux/man-pages/man2/getrlimit.2.html
                RLIMIT_AS,
                RLIMIT_CORE,
                RLIMIT_CPU,
                RLIMIT_DATA,
                RLIMIT_FSIZE,
                RLIMIT_LOCKS,
                RLIMIT_MEMLOCK,
                RLIMIT_MSGQUEUE,
                RLIMIT_NICE,
                RLIMIT_NOFILE,
                RLIMIT_NPROC,
                RLIMIT_RSS,
                RLIMIT_RTPRIO,
                RLIMIT_RTTIME,
                RLIMIT_SIGPENDING,
                RLIMIT_STACK,
            }
        }
    }else if #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "linux", // target_env != "gnu"
    ))]{
        use libc::{c_int, rlimit, RLIM_INFINITY};

        libc_enum! {
            #[repr(i32)]
            pub enum Resource {
                /// See detail of each Resource https://man7.org/linux/man-pages/man2/getrlimit.2.html
                /// BSD specific Resource https://www.freebsd.org/cgi/man.cgi?query=setrlimit
                #[cfg(not(any(target_os = "netbsd", target_os = "freebsd")))]
                RLIMIT_AS,
                RLIMIT_CORE,
                RLIMIT_CPU,
                RLIMIT_DATA,
                RLIMIT_FSIZE,
                RLIMIT_NOFILE,
                RLIMIT_STACK,

                // platform specific
                #[cfg(target_os = "freebsd")]
                RLIMIT_KQUEUES,

                #[cfg(any(target_os = "android", target_os = "linux"))]
                RLIMIT_LOCKS,

                #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
                RLIMIT_MEMLOCK,

                #[cfg(any(target_os = "android", target_os = "linux"))]
                RLIMIT_MSGQUEUE,

                #[cfg(any(target_os = "android", target_os = "linux"))]
                RLIMIT_NICE,

                #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
                RLIMIT_NPROC,

                #[cfg(target_os = "freebsd")]
                RLIMIT_NPTS,

                #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
                RLIMIT_RSS,

                #[cfg(any(target_os = "android", target_os = "linux"))]
                RLIMIT_RTPRIO,

                #[cfg(any(target_os = "linux"))]
                RLIMIT_RTTIME,

                #[cfg(any(target_os = "android", target_os = "linux"))]
                RLIMIT_SIGPENDING,

                #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
                RLIMIT_SBSIZE,

                #[cfg(target_os = "freebsd")]
                RLIMIT_SWAP,

                #[cfg(target_os = "freebsd")]
                RLIMIT_VMEM,
            }
        }
    }else{
        // unkown os
    }
}

/// Get the current processes resource limits
///
/// A value of `None` indicates that there's no limit.
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to get the limits of.
///
/// # Examples
///
/// ```
/// # use nix::sys::resource::{getrlimit, Resource};
///
/// let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
/// println!("current soft_limit: {:?}", soft_limit);
/// println!("current hard_limit: {:?}", hard_limit);
/// ```
///
/// # References
///
/// [getrlimit(2)](https://linux.die.net/man/2/getrlimit)
///
/// [`Resource`]: enum.Resource.html
pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut old_rlim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    cfg_if! {
        if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
            let res = unsafe { libc::getrlimit(resource as __rlimit_resource_t, &mut old_rlim) };
        }else if #[cfg(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "linux", // target_env != "gnu"
        ))]{
            let res = unsafe { libc::getrlimit(resource as c_int, &mut old_rlim) };
        }
    }

    Errno::result(res).map(|_| {
        (
            Some(old_rlim.rlim_cur).filter(|x| *x != RLIM_INFINITY),
            Some(old_rlim.rlim_max).filter(|x| *x != RLIM_INFINITY),
        )
    })
}

/// Set the current processes resource limits
///
/// A value of `None` indicates that there's no limit.
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to set the limits of.
/// * `soft_limit`: The value that the kernel enforces for the corresponding
///   resource. Note: `None` input will be replaced by constant `RLIM_INFINITY`.
/// * `hard_limit`: The ceiling for the soft limit. Must be lower or equal to
///   the current hard limit for non-root users. Note: `None` input will be
///   replaced by constant `RLIM_INFINITY`.
///
/// # Examples
///
/// ```no_run
/// # use nix::sys::resource::{setrlimit, Resource};
///
/// let soft_limit = Some(1024);
/// let hard_limit = None;
/// setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();
/// ```
///
/// # References
///
/// [setrlimit(2)](https://linux.die.net/man/2/setrlimit)
///
/// [`Resource`]: enum.Resource.html
pub fn setrlimit(
    resource: Resource,
    soft_limit: Option<rlim_t>,
    hard_limit: Option<rlim_t>,
) -> Result<()> {
    let new_rlim = rlimit {
        rlim_cur: soft_limit.unwrap_or(RLIM_INFINITY),
        rlim_max: hard_limit.unwrap_or(RLIM_INFINITY),
    };
    cfg_if! {
        if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
            // the below implementation is mimicing the similar implementation in golang
            // https://go-review.googlesource.com/c/sys/+/230478/2/unix/syscall_linux_arm64.go#176
            // seems for some of the architectures, we prefer to use prlimit instead of {g,s}etrlimit
            let res = unsafe { libc::prlimit(0, resource as __rlimit_resource_t, &new_rlim as *const _, std::ptr::null_mut()) };
            if res == -1 {
                return Errno::result(res).map(|_| ());
            }

            let res = unsafe { libc::setrlimit(resource as __rlimit_resource_t, &new_rlim as *const _) };

        }else if #[cfg(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "dragonfly",
            target_os = "bitrig",
            target_os = "linux", // target_env != "gnu"
        ))]{
            let res = unsafe { libc::setrlimit(resource as c_int, &new_rlim as *const _) };
        }
    }

    Errno::result(res).map(|_| ())
}
