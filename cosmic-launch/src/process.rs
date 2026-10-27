// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[cfg(all(feature = "smol", not(feature = "tokio")))]
use smol::io::AsyncReadExt;
use std::io;
use std::os::fd::OwnedFd;
use std::process::{Command, Stdio};
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;

#[cfg(feature = "tokio")]
async fn read_from_pipe(read: OwnedFd) -> Option<u32> {
    let mut read = tokio::net::unix::pipe::Receiver::from_owned_fd(read).unwrap();
    read.read_u32().await.ok()
}

#[cfg(all(feature = "smol", not(feature = "tokio")))]
async fn read_from_pipe(read: OwnedFd) -> Option<u32> {
    let mut read = smol::Async::new(std::fs::File::from(read)).unwrap();
    let mut bytes = [0; 4];
    read.read_exact(&mut bytes).await.ok()?;
    Some(u32::from_be_bytes(bytes));
}

#[cfg(not(any(feature = "tokio", feature = "smol")))]
async fn read_from_pipe(read: OwnedFd) -> Option<u32> {
    use rustix::fd::AsFd;
    let mut bytes = [0u8; 4];
    rustix::io::read(&read, &mut bytes).ok()?;
    Some(u32::from_be_bytes(bytes));
}

/// Performs a double fork with setsid to spawn and detach a command.
pub async fn spawn(mut command: Command) -> Option<u32> {
    // NOTE: Windows platform is not supported
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Handle Linux
    #[cfg(all(unix, not(target_os = "macos")))]
    let Ok((read, write)) = rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC) else {
        return None;
    };

    // Handle macOS
    #[cfg(target_os = "macos")]
    let Ok((read, write)) = rustix::pipe::pipe() else {
        return None;
    };

    match unsafe { libc::fork() } {
        // Parent process
        fork_pid @ 1..=i32::MAX => {
            // Drop copy of write end, then read PID from pipe
            drop(write);
            let pid = read_from_pipe(read).await;
            _ = rustix::process::waitpid(
                rustix::process::Pid::from_raw(fork_pid),
                rustix::process::WaitOptions::empty(),
            );
            pid
        }

        // Child process
        0 => {
            let _res = rustix::process::setsid();
            let exit_status = if let Ok(child) = command.spawn() {
                // Write PID to pipe
                let _ = rustix::io::write(write, &child.id().to_be_bytes());
                0
            } else {
                1
            };

            // # Safety
            // Required for child fork to exit without affecting the parent.
            unsafe {
                libc::_exit(exit_status);
            }
        }

        ..=-1 => {
            let error = io::Error::last_os_error();
            tracing::error!(?error, "failed to fork and spawn command");

            None
        }
    }
}
