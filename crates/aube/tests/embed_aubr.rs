#![cfg(unix)]

use std::process::{Command, Stdio};

static HOST: aube_util::Embedder = aube_util::Embedder {
    runtime_switching: false,
    self_engines_check: false,
    self_update_enabled: false,
    ..aube_util::identity::AUBE
};

#[test]
fn embedded_aubr_returns_to_host() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("package.json"),
        r#"{"name":"host-test","scripts":{"fail":"exit 7"}}"#,
    )
    .unwrap();

    for entrypoint in ["args", "defaults"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "embedded_aubr_child", "--nocapture"])
            .env("AUBE_TEST_EMBEDDED_AUBR", entrypoint)
            .env("HOME", project.path())
            .env("XDG_CACHE_HOME", project.path().join("cache"))
            .env("XDG_DATA_HOME", project.path().join("data"))
            .current_dir(project.path())
            .output()
            .unwrap();
        assert!(output.status.success(), "{entrypoint}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("host resumed"));
    }
}

#[test]
fn standalone_aubr_replaces_process() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("package.json"),
        r#"{"name":"standalone-test","scripts":{"pid":"echo $$"}}"#,
    )
    .unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_aubr"))
        .args(["--no-install", "pid"])
        .env("HOME", project.path())
        .env("XDG_CACHE_HOME", project.path().join("cache"))
        .env("XDG_DATA_HOME", project.path().join("data"))
        .current_dir(project.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        pid.to_string()
    );
}

#[test]
fn embedded_aubr_child() {
    let Ok(entrypoint) = std::env::var("AUBE_TEST_EMBEDDED_AUBR") else {
        return;
    };
    let args = ["aubr", "--no-install", "fail"];
    let code = if entrypoint == "defaults" {
        aube::cli_main_with_defaults_from_args(&HOST, Vec::new(), args)
    } else {
        aube::cli_main_from_args(&HOST, args)
    };
    assert_eq!(code, 7);
    println!("host resumed");
}
