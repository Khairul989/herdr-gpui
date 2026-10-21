//! A host shell puts Linuxbrew on `PATH` only where it is installed. On macOS
//! `/home` is an automount: every command found through a missing directory
//! there costs a directory-service query, and the local port scan runs
//! dozens of commands every few seconds through this shell.
use crate::usage::probe::Shell;
use std::{path::Path, time::Duration};

const LINUXBREW: &str = "/home/linuxbrew/.linuxbrew/bin";

#[test]
fn linuxbrew_is_on_the_shell_path_only_where_installed() {
    let home = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-s")
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut shell = Shell::start(command).unwrap();
    let output = shell
        .run("printf '%s' \"$PATH\"", Duration::from_secs(5))
        .unwrap();
    assert!(output.success);
    let dirs: Vec<&str> = output.stdout.split(':').collect();
    assert_eq!(
        dirs.contains(&LINUXBREW),
        Path::new(LINUXBREW).is_dir(),
        "PATH: {}",
        output.stdout
    );
    // The rest of the prelude still leads, ahead of the inherited PATH.
    let position = |dir: &str| dirs.iter().position(|entry| *entry == dir);
    assert!(position("/usr/local/bin") < position("/usr/bin"));
    // Where installed, linuxbrew keeps its place: after `/usr/local`, ahead
    // of the inherited PATH.
    if let Some(linuxbrew) = position(LINUXBREW) {
        assert!(position("/usr/local/bin") < Some(linuxbrew));
        assert!(Some(linuxbrew) < position("/usr/bin"));
    }
    assert_eq!(dirs.last(), Some(&"/bin"));
}
