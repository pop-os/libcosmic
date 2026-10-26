// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod entry;
pub use entry::EntryLauncher;

mod executable;
pub use executable::{CustomExecutable, Executable};

pub mod gpu;
pub mod process;

#[cfg(feature = "systemd")]
pub mod systemd;

use bstr::{BString, ByteSlice, ByteVec};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

error_set::error_set! {
    Error := {
        #[display("failed to expand arguments")]
        ArgumentExpansion,
        #[display("fork and spawn failed")]
        ForkAndSpawn,
    }
}

/// Spawn applications using a double-fork with setsid.
#[cfg(not(feature = "systemd"))]
pub async fn spawn<E: Executable>(
    executable: &E,
    action: Option<&str>,
    gpus: &[gpu::Descriptor],
    gpu_pref_override: Option<gpu::Preference>,
    mut env_vars: Vec<(String, String)>,
    paths: &[impl AsRef<OsStr> + Sized],
) -> Result<(), Error> {
    if let Some(pos) = gpu_pref_override
        .map(|pref| pref.select(gpus))
        .unwrap_or_else(|| executable.gpu_preference().select(gpus))
    {
        env_vars.extend(gpus[pos].environment.clone());
    }

    let commands =
        expand_cmdline_arguments(executable, action, paths).ok_or(Error::ArgumentExpansion)?;

    for argv in commands {
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.envs(env_vars.clone());
        crate::process::spawn(cmd)
            .await
            .ok_or(Error::ForkAndSpawn)
            .map(|_| ())?;
    }

    Ok(())
}

/// Spawn applications as transient `.service` units with `org.systemd1.Manager`.
///
/// # Note
/// Falls back to a double-fork with setsid if that is not possible.
#[cfg(feature = "systemd")]
pub async fn spawn<E: Executable>(
    conn: &zbus::Connection,
    executable: &E,
    action: Option<&str>,
    gpus: &[gpu::Descriptor],
    gpu_pref_override: Option<gpu::Preference>,
    mut env_vars: Vec<(String, String)>,
    paths: &[impl AsRef<OsStr> + Sized],
) -> Result<(), Error> {
    if let Some(pos) = gpu_pref_override
        .map(|pref| pref.select(gpus))
        .unwrap_or_else(|| executable.gpu_preference().select(gpus))
    {
        env_vars.extend(gpus[pos].environment.clone());
    }

    let commands =
        expand_cmdline_arguments(executable, action, paths).ok_or(Error::ArgumentExpansion)?;

    let systemd_env_vars: Vec<String> = env_vars
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    for argv in commands {
        match systemd::start_transient_unit(conn, executable, &systemd_env_vars, &argv).await {
            Ok(_) => return Ok(()),
            Err(why) => {
                tracing::error!(
                    ?why,
                    "systemd::start_transient_unit failed. Falling back to fork-and-spawn"
                );
            }
        }

        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.envs(env_vars.clone());
        crate::process::spawn(cmd)
            .await
            .ok_or(Error::ForkAndSpawn)
            .map(|_| ())?;
    }

    Ok(())
}

fn expand_cmdline_arguments<E: Executable>(
    executable: &E,
    action: Option<&str>,
    opt_paths: &[impl AsRef<OsStr> + Sized],
) -> Option<Vec<Vec<String>>> {
    let term_exec;

    let exec_str = if executable.is_terminal() {
        let term = cosmic_settings_config::shortcuts::context()
            .ok()
            .and_then(|config| {
                cosmic_settings_config::shortcuts::system_actions(&config)
                    .get(&cosmic_settings_config::shortcuts::action::System::Terminal)
                    .cloned()
            })
            .unwrap_or_else(|| String::from("cosmic-term"));

        term_exec = format!("{term} -e {}", executable.exec(action));
        &term_exec
    } else {
        executable.exec(action)
    };

    let arguments = shlex::split(exec_str)?;

    let exec = match arguments.first() {
        Some(exec) if !exec.contains('=') => exec,
        _ => return None,
    };

    let mut commands = Vec::new();

    let paths = opt_paths
        .iter()
        .map(AsRef::as_ref)
        .map(Some)
        // Add a single `None` if no path was given.
        .chain(std::iter::repeat_n(
            None,
            if opt_paths.is_empty() { 1 } else { 0 },
        ));

    for path in paths {
        let mut batch_process = false;
        let mut args = Vec::with_capacity(arguments.len());
        let mut field_code_used = false;

        for argument in arguments.iter().skip(1) {
            let mut new_argument = BString::new(Vec::with_capacity(argument.capacity()));
            let mut chars = argument.chars();
            while let Some(char) = chars.next() {
                // https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
                if char == '%' {
                    match chars.next() {
                        Some('%') => new_argument.push_char(char),
                        Some('c') => new_argument.push_str(executable.name().as_ref()),
                        Some('k') => {
                            if let Some(path) = executable.path() {
                                new_argument.push_str(path.as_bytes());
                            }
                        }

                        // %f and %u behave the same in a file manager.
                        Some('f' | 'u') => {
                            if let Some(path) = path
                                && !field_code_used
                            {
                                // TODO: files on remote file systems should be copied to a temporary local file.
                                batch_process = true;
                                field_code_used = true;
                                new_argument.push_str(path.as_bytes());
                            }
                        }

                        // %F and %U behave the same in a file manager.
                        Some('F') | Some('U') if !field_code_used && new_argument.is_empty() => {
                            field_code_used = true;
                            for path in opt_paths.iter().map(AsRef::as_ref) {
                                args.push(BString::new(path.as_bytes().to_owned()));
                            }
                        }

                        _ => (),
                    }
                } else {
                    new_argument.push_char(char);
                }
            }

            if !new_argument.is_empty() {
                args.push(new_argument);
            }
        }

        let mut command = vec![exec.to_owned()];

        for arg in args {
            match arg.to_str() {
                Ok(arg) => {
                    command.push(arg.to_owned());
                }
                Err(_) => {
                    tracing::error!("invalid string encoding in command");
                    return None;
                }
            }
        }

        commands.push(command);

        if !batch_process {
            break;
        }
    }

    Some(commands)
}
