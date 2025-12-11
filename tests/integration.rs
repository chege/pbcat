use assert_cmd::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn end_to_end_with_separator_header_and_sort() {
    let tmp = tempdir().unwrap();
    let out = tmp.path().join("clipboard.txt");

    let a = write(tmp.path(), "b.txt", "Bravo");
    let b = write(tmp.path(), "a.txt", "Alpha");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("pbcat"));
    cmd.env("PBCAT_CLIPBOARD_FILE", &out)
        .arg("--sort")
        .arg("name")
        .arg("-H")
        .arg("-s")
        .arg("\n---\n")
        .arg(a.clone())
        .arg(b.clone());

    cmd.assert().success();

    let expected = format!(
        "== {} ==\nAlpha\n---\n== {} ==\nBravo",
        canonical(&b).display(),
        canonical(&a).display()
    );

    let got = fs::read_to_string(&out).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn list_mode_prints_files_and_skips_clipboard() {
    let tmp = tempdir().unwrap();
    let out = tmp.path().join("clipboard.txt");
    let a = write(tmp.path(), "one.txt", "one");
    let b = write(tmp.path(), "two.txt", "two");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("pbcat"));
    cmd.env("PBCAT_CLIPBOARD_FILE", &out)
        .arg("--list")
        .arg(&a)
        .arg(&b);

    cmd.assert()
        .success()
        .stdout(contains(canonical(&a).to_string_lossy().as_ref()))
        .stdout(contains(canonical(&b).to_string_lossy().as_ref()))
        .stdout(contains("Listed 2 files"));

    assert!(
        fs::metadata(&out).is_err(),
        "clipboard file should not be written in list mode"
    );
}

fn write(base: &Path, name: &str, contents: &str) -> PathBuf {
    let path = base.join(name);
    fs::write(&path, contents).unwrap();
    path
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap()
}
