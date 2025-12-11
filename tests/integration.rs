use assert_cmd::prelude::*;
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

fn write(base: &Path, name: &str, contents: &str) -> PathBuf {
    let path = base.join(name);
    fs::write(&path, contents).unwrap();
    path
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap()
}
