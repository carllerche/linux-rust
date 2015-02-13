#![feature(core, std_misc)]

extern crate nix;
extern crate rand;

#[cfg(test)]
mod test {
    use nix::unistd::{writev, readv, Iovec, pipe, close, read, write, dup,
                      execve};
    use std::cmp::min;
    use std::ffi::CString;
    use std::iter::repeat;
    use rand::{thread_rng, Rng};

    use nix::unistd::{fork};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::Fork::{Parent, Child};

    #[test]
    fn test_writev() {
        let mut to_write = Vec::with_capacity(16 * 128);
        for _ in 0..16 {
            let s: String = thread_rng().gen_ascii_chars().take(128).collect();
            let b = s.as_bytes();
            to_write.extend(b.iter().map(|x| x.clone()));
        }
        // Allocate and fill iovecs
        let mut iovecs = Vec::new();
        let mut consumed = 0;
        while consumed < to_write.len() {
            let left = to_write.len() - consumed;
            let slice_len = if left < 64 { left } else { thread_rng().gen_range(64, min(256, left)) };
            let b = &to_write[consumed..consumed+slice_len];
            iovecs.push(Iovec::from_slice(b));
            consumed += slice_len;
        }
        let pipe_res = pipe();
        assert!(pipe_res.is_ok());
        let (reader, writer) = pipe_res.ok().unwrap();
        // FileDesc will close its filedesc (reader).
        let mut read_buf: Vec<u8> = repeat(0u8).take(128 * 16).collect();
        // Blocking io, should write all data.
        let write_res = writev(writer, iovecs.as_slice());
        // Successful write
        assert!(write_res.is_ok());
        let written = write_res.ok().unwrap();
        // Check whether we written all data
        assert_eq!(to_write.len(), written);
        let read_res = read(reader, read_buf.as_mut_slice());
        // Successful read
        assert!(read_res.is_ok());
        let read = read_res.ok().unwrap() as usize;
        // Check we have read as much as we written
        assert_eq!(read, written);
        // Check equality of written and read data
        assert_eq!(to_write.as_slice(), read_buf.as_slice());
        let close_res = close(writer);
        assert!(close_res.is_ok());
        let close_res = close(reader);
        assert!(close_res.is_ok());
    }

    #[test]
    fn test_readv() {
        let s:String = thread_rng().gen_ascii_chars().take(128).collect();
        let to_write = s.as_bytes().to_vec();
        let mut storage = Vec::new();
        let mut allocated = 0;
        while allocated < to_write.len() {
            let left = to_write.len() - allocated;
            let vec_len = if left < 64 { left } else { thread_rng().gen_range(64, min(256, left)) };
            let v: Vec<u8> = repeat(0u8).take(vec_len).collect();
            storage.push(v);
            allocated += vec_len;
        }
        let mut iovecs = Vec::with_capacity(storage.len());
        for v in storage.iter_mut() {
            iovecs.push(Iovec::from_mut_slice(v.as_mut_slice()));
        }
        let pipe_res = pipe();
        assert!(pipe_res.is_ok());
        let (reader, writer) = pipe_res.ok().unwrap();
        // Blocking io, should write all data.
        let write_res = write(writer, to_write.as_slice());
        // Successful write
        assert!(write_res.is_ok());
        let read_res = readv(reader, iovecs.as_mut_slice());
        assert!(read_res.is_ok());
        let read = read_res.ok().unwrap();
        // Check whether we've read all data
        assert_eq!(to_write.len(), read);
        // Cccumulate data from iovecs
        let mut read_buf = Vec::with_capacity(to_write.len());
        for iovec in iovecs.iter() {
            read_buf.extend(iovec.as_slice().iter().map(|x| x.clone()));
        }
        // Check whether iovecs contain all written data
        assert_eq!(read_buf.len(), to_write.len());
        // Check equality of written and read data
        assert_eq!(read_buf.as_slice(), to_write.as_slice());
        let close_res = close(reader);
        assert!(close_res.is_ok());
        let close_res = close(writer);
        assert!(close_res.is_ok());
    }


    #[test]
    fn test_fork_and_waitpid() {
        let pid = fork();
        match pid {
          Ok(Child) => {} // ignore child here
          Ok(Parent(child_pid)) => {
              // assert that child was created and pid > 0
              assert!(child_pid > 0);
              let wait_status = waitpid(child_pid, None);
              match wait_status {
                  // assert that waitpid returned correct status and the pid is the one of the child
                  Ok(WaitStatus::Exited(pid_t)) =>  assert!(pid_t == child_pid),

                  // panic, must never happen
                  Ok(WaitStatus::StillAlive) => panic!("Child still alive, should never happen"),

                  // panic, waitpid should never fail
                  Err(_) => panic!("Error: waitpid Failed")
              }

          },
          // panic, fork should never fail unless there is a serious problem with the OS
          Err(_) => panic!("Error: Fork Failed")
        }
    }

    #[test]
    fn test_execve() {
        // The `exec`d process will write to `writer`, and we'll read that
        // data from `reader`.
        let (reader, writer) = pipe().unwrap();

        match fork().unwrap() {
            Child => {
                // Close stdout.
                close(1).unwrap();
                // Make `writer` be the stdout of the new process.
                dup(writer).unwrap();
                // exec!
                execve(&CString::from_slice(b"/bin/sh"),
                       &[CString::from_slice(b""),
                         CString::from_slice(b"-c"),
                         CString::from_slice(b"echo nix!!! && env")],
                       &[CString::from_slice(b"foo=bar"),
                         CString::from_slice(b"baz=quux")]).unwrap();
            },
            Parent(child_pid) => {
                // Wait for the child to exit.
                waitpid(child_pid, None).unwrap();
                // Read 1024 bytes.
                let mut buf = [0u8; 1024];
                read(reader, &mut buf).unwrap();
                // It should contain the things we printed using `/bin/sh`.
                let string = String::from_utf8_lossy(&buf);
                assert!(string.contains("nix!!!"));
                assert!(string.contains("foo=bar"));
                assert!(string.contains("baz=quux"));
            }
        }
    }
}
