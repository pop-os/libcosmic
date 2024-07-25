use std::fs::File;
use std::io;
use std::process::{exit, Command, Stdio};
use tokio::io::AsyncReadExt;

/// Performs a double fork with setsid to spawn and detach a command.
pub async fn spawn(mut command: Command) -> Option<u32> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let Ok((read, write)) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC) else {
        return None;
    };

    match unsafe { libc::fork() } {
        // Parent process
        1.. => {
            drop(write);
            // Read PID from pipe
            let mut read = tokio::net::unix::pipe::Receiver::from_owned_fd(read).unwrap();
            read.read_u32().await.ok()
        }

        // Child process
        0 => {
            let _res = rustix::process::setsid();
            if let Ok(child) = command.spawn() {
                // Write PID to pipe
                let _ = rustix::io::write(write, &child.id().to_be_bytes());
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
