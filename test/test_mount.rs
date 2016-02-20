// Impelmentation note: to allow unprivileged users to run it, this test makes
// use of user and mount namespaces. On systems that allow unprivileged user
// namespaces (Linux >= 3.8 compiled with CONFIG_USER_NS), the test should run
// without root.

#[macro_use]
extern crate nix;
extern crate libc;
extern crate tempdir;

#[cfg(target_os = "linux")]
mod test_mount {
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::ffi::CStr;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use libc::{self, EACCES, EROFS};

    use nix::cstr::ToCString;
    use nix::mount::{mount, umount, MsFlags, MS_BIND, MS_RDONLY, MS_NOEXEC};
    use nix::sched::{unshare, CLONE_NEWNS, CLONE_NEWUSER};
    use nix::sys::stat::{self, S_IRWXU, S_IRWXG, S_IRWXO, S_IXUSR, S_IXGRP, S_IXOTH};

    use tempdir::TempDir;

    static SCRIPT_CONTENTS: &'static [u8] = b"#!/bin/sh
exit 23";

    const EXPECTED_STATUS: i32 = 23;

    const NONE: Option<&'static CStr> = None;

    pub fn test_mount_tmpfs_without_flags_allows_rwx() {
        let tempdir = TempDir::new("nix-test_mount")
                          .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
        let temppath = tempdir.path().to_cstring().unwrap();

        mount(NONE,
              &temppath,
              Some(cstr!("tmpfs")),
              MsFlags::empty(),
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        let test_path = tempdir.path().join("test");

        // Verify write.
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode((S_IRWXU | S_IRWXG | S_IRWXO).bits())
            .open(&test_path)
            .and_then(|mut f| f.write(SCRIPT_CONTENTS))
            .unwrap_or_else(|e| panic!("write failed: {}", e));

        // Verify read.
        let mut buf = Vec::new();
        File::open(&test_path)
            .and_then(|mut f| f.read_to_end(&mut buf))
            .unwrap_or_else(|e| panic!("read failed: {}", e));
        assert_eq!(buf, SCRIPT_CONTENTS);

        // Verify execute.
        assert_eq!(EXPECTED_STATUS,
                   Command::new(&test_path)
                       .status()
                       .unwrap_or_else(|e| panic!("exec failed: {}", e))
                       .code()
                       .unwrap_or_else(|| panic!("child killed by signal")));

        umount(&temppath).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_rdonly_disallows_write() {
        let tempdir = TempDir::new("nix-test_mount")
                          .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
        let temppath = tempdir.path().to_cstring().unwrap();

        mount(NONE,
              &temppath,
              Some(cstr!("tmpfs")),
              MS_RDONLY,
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        // EROFS: Read-only file system
        assert_eq!(EROFS as i32,
                   File::create(tempdir.path().join("test")).unwrap_err().raw_os_error().unwrap());

        umount(&temppath).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_noexec_disallows_exec() {
        let tempdir = TempDir::new("nix-test_mount")
                          .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
        let temppath = tempdir.path().to_cstring().unwrap();

        mount(NONE,
              &temppath,
              Some(cstr!("tmpfs")),
              MS_NOEXEC,
              NONE)
            .unwrap_or_else(|e| panic!("mount failed: {}", e));

        let test_path = tempdir.path().join("test");

        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode((S_IRWXU | S_IRWXG | S_IRWXO).bits())
            .open(&test_path)
            .and_then(|mut f| f.write(SCRIPT_CONTENTS))
            .unwrap_or_else(|e| panic!("write failed: {}", e));

        // Verify that we cannot execute despite a+x permissions being set.
        let mode = stat::Mode::from_bits_truncate(fs::metadata(&test_path)
                                                      .map(|md| md.permissions().mode())
                                                      .unwrap_or_else(|e| {
                                                          panic!("metadata failed: {}", e)
                                                      }));

        assert!(mode.contains(S_IXUSR | S_IXGRP | S_IXOTH),
                "{:?} did not have execute permissions",
                &test_path);

        // EACCES: Permission denied
        assert_eq!(EACCES as i32,
                   Command::new(&test_path).status().unwrap_err().raw_os_error().unwrap());

        umount(&temppath).unwrap_or_else(|e| panic!("umount failed: {}", e));
    }

    pub fn test_mount_bind() {
        use std::env;
        if env::var("CI").is_ok() && env::var("TRAVIS").is_ok() {
            print!("Travis does not allow bind mounts, skipping.");
            return;
        }

        let tempdir = TempDir::new("nix-test_mount")
                          .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
        let temppath = tempdir.path().to_cstring().unwrap();
        let file_name = "test";

        {
            let mount_point = TempDir::new("nix-test_mount")
                                  .unwrap_or_else(|e| panic!("tempdir failed: {}", e));
            let mount_path = mount_point.path().to_cstring().unwrap();

            mount(Some(&temppath),
                  &mount_path,
                  NONE,
                  MS_BIND,
                  NONE)
                .unwrap_or_else(|e| panic!("mount failed: {}", e));

            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .mode((S_IRWXU | S_IRWXG | S_IRWXO).bits())
                .open(mount_point.path().join(file_name))
                .and_then(|mut f| f.write(SCRIPT_CONTENTS))
                .unwrap_or_else(|e| panic!("write failed: {}", e));

            umount(&mount_path).unwrap_or_else(|e| panic!("umount failed: {}", e));
        }

        // Verify the file written in the mount shows up in source directory, even
        // after unmounting.

        let mut buf = Vec::new();
        File::open(tempdir.path().join(file_name))
            .and_then(|mut f| f.read_to_end(&mut buf))
            .unwrap_or_else(|e| panic!("read failed: {}", e));
        assert_eq!(buf, SCRIPT_CONTENTS);
    }

    pub fn setup_namespaces() {
        // Hold on to the uid in the parent namespace.
        let uid = unsafe { libc::getuid() };

        unshare(CLONE_NEWNS | CLONE_NEWUSER).unwrap_or_else(|e| {
            panic!("unshare failed: {}. Are unprivileged user namespaces available?",
                   e)
        });

        // Map user as uid 1000.
        fs::OpenOptions::new()
            .write(true)
            .open("/proc/self/uid_map")
            .and_then(|mut f| f.write(format!("1000 {} 1\n", uid).as_bytes()))
            .unwrap_or_else(|e| panic!("could not write uid map: {}", e));
    }
}


// Test runner

/// Mimic normal test output (hackishly).
macro_rules! run_tests {
    ( $($test_fn:ident),* ) => {{
        print!("\n");

        $(
            print!("test test_mount::{} ... ", stringify!($test_fn));
            $test_fn();
            print!("ok\n");
        )*

        print!("\n");
    }}
}

#[cfg(target_os = "linux")]
fn main() {
    use test_mount::{setup_namespaces, test_mount_tmpfs_without_flags_allows_rwx,
                     test_mount_rdonly_disallows_write, test_mount_noexec_disallows_exec,
                     test_mount_bind};
    setup_namespaces();

    run_tests!(test_mount_tmpfs_without_flags_allows_rwx,
               test_mount_rdonly_disallows_write,
               test_mount_noexec_disallows_exec,
               test_mount_bind);
}

#[cfg(not(target_os = "linux"))]
fn main() {}
