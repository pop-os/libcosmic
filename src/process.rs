use std::fs::File;
use std::io;
use std::process::{exit, Command, Stdio};

/// Performs a double fork with setsid to spawn and detach a command.
pub fn spawn(mut command: Command) -> Option<u32> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let Ok((read, write)) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC) else {
        return None;
    };

    match unsafe { libc::fork() } {
        // Parent process
        child @ 1.. => {
            let child = rustix::process::Pid::from_raw(child).unwrap();
            let _res = rustix::process::waitpid(Some(child), rustix::process::WaitOptions::empty());
            // Read PID from pipe
            let mut bytes = [0; 4];
            if rustix::io::read(read, &mut bytes) == Ok(4) {
                Some(u32::from_ne_bytes(bytes))
            } else {
                None
            }
        }

        // Child process
        0 => {
            let _res = rustix::process::setsid();
            if let Ok(child) = command.spawn() {
                // Write PID to pipe
                let _ = rustix::io::write(write, &child.id().to_ne_bytes());
            }

            exit(0)
        }

        ..=-1 => {
            println!(
                "failed to fork and spawn command: {}",
                io::Error::last_os_error()
            );

            None
        }
    }
}
