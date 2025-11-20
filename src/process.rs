// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[cfg(all(feature = "smol", not(feature = "tokio")))]
use smol::io::AsyncReadExt;
use std::io;
use std::os::fd::OwnedFd;
use std::process::{Command, Stdio, exit};
#[cfg(feature = "tokio")]
use tokio::io::AsyncReadExt;

async fn read_from_pipe(read: OwnedFd) -> Option<u32> {
    #[cfg(feature = "tokio")]
    {
        let mut read = tokio::net::unix::pipe::Receiver::from_owned_fd(read).unwrap();
        return read.read_u32().await.ok();
    }

    #[cfg(all(feature = "smol", not(feature = "tokio")))]
    {
        let mut read = smol::Async::new(std::fs::File::from(read)).unwrap();
        let mut bytes = [0; 4];
        read.read_exact(&mut bytes).await.ok()?;
        return Some(u32::from_be_bytes(bytes));
    }

    #[cfg(not(any(feature = "tokio", feature = "smol")))]
    {
        use rustix::fd::AsFd;
        let mut bytes = [0u8; 4];
        rustix::io::read(&read, &mut bytes).ok()?;
        return Some(u32::from_be_bytes(bytes));
    }
}

/// Performs a double fork with setsid to spawn and detach a command.
#[cold]
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
        1.. => {
            // Drop copy of write end, then read PID from pipe
            drop(write);
            let pid = read_from_pipe(read).await;
            // wait to prevent zombie
            _ = rustix::process::wait(rustix::process::WaitOptions::empty());
            pid
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
