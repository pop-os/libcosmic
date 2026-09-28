//! Isolate process-global configuration and session-bus access in test subprocesses.

pub fn run(name: &str, test: impl FnOnce()) {
    run_case(name, "", test);
}

pub fn run_case(name: &str, case: &str, test: impl FnOnce()) {
    const COMPLETION: &str = "COSMIC_TEST_COMPLETION";
    if let Some(path) = std::env::var_os(COMPLETION) {
        if std::env::var("COSMIC_TEST_CASE").unwrap() == case {
            test();
            std::fs::write(path, b"complete").unwrap();
        }
        return;
    }

    let directory = tempfile::tempdir().unwrap();
    let completion = directory.path().join("complete");
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(COMPLETION, &completion)
        .env("COSMIC_TEST_CASE", case)
        // A valid but nonexistent private socket prevents contact with the user's bus.
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!(
                "unix:path={}",
                directory.path().join("no-session-bus").display()
            ),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(completion).unwrap(),
        b"complete",
        "the selected test must have run to completion"
    );
}
