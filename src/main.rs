use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!(
                "Copied {} file{} ({} bytes) to clipboard",
                summary.files,
                if summary.files == 1 { "" } else { "s" },
                summary.bytes
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

struct CopySummary {
    files: usize,
    bytes: usize,
}

fn run() -> Result<CopySummary, Box<dyn Error>> {
    let args: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();
    if args.is_empty() {
        return Err("Usage: pbcat <file> [file ...]".into());
    }

    let contents = gather_contents(&args)?;
    copy_to_clipboard(&contents)?;

    Ok(CopySummary {
        files: args.len(),
        bytes: contents.as_bytes().len(),
    })
}

fn gather_contents(paths: &[PathBuf]) -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();

    for path in paths {
        let metadata = fs::metadata(path).map_err(|e| format!("{}: {}", display(path), e))?;
        if !metadata.is_file() {
            return Err(format!("{}: not a file", display(path)).into());
        }

        let bytes = fs::read(path).map_err(|e| format!("{}: {}", display(path), e))?;
        let text = String::from_utf8(bytes)
            .map_err(|_| format!("{}: not valid UTF-8", display(path)))?;
        buffer.push_str(&text);
    }

    Ok(buffer)
}

fn copy_to_clipboard(data: &str) -> Result<(), Box<dyn Error>> {
    for tool in preferred_clipboard_tools() {
        if attempt_copy(tool, data).is_ok() {
            return Ok(());
        }
    }

    Err("No supported clipboard utility found (tried pbcopy/wl-copy/xclip/xsel)".into())
}

struct ClipboardTool {
    program: &'static str,
    args: &'static [&'static str],
}

fn attempt_copy(tool: ClipboardTool, data: &str) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(tool.program)
        .args(tool.args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {}", tool.program, e))?;

    {
        let mut stdin = child.stdin.take().ok_or("failed to open stdin")?;
        stdin.write_all(data.as_bytes())?;
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with status {}", tool.program, status).into())
    }
}

#[cfg(target_os = "macos")]
fn preferred_clipboard_tools() -> Vec<ClipboardTool> {
    vec![ClipboardTool {
        program: "pbcopy",
        args: &[],
    }]
}

#[cfg(target_os = "linux")]
fn preferred_clipboard_tools() -> Vec<ClipboardTool> {
    vec![
        ClipboardTool {
            program: "wl-copy",
            args: &[],
        },
        ClipboardTool {
            program: "xclip",
            args: &["-selection", "clipboard"],
        },
        ClipboardTool {
            program: "xsel",
            args: &["--clipboard", "--input"],
        },
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn preferred_clipboard_tools() -> Vec<ClipboardTool> {
    Vec::new()
}

fn display(path: &PathBuf) -> String {
    path.as_os_str().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn gathers_files_in_order() {
        let dir = tempdir();
        let first = dir.join("a.txt");
        let second = dir.join("b.txt");

        write_file(&first, "hello").unwrap();
        write_file(&second, "world").unwrap();

        let combined = gather_contents(&[first, second]).unwrap();
        assert_eq!(combined, "helloworld");
    }

    #[test]
    fn rejects_non_files() {
        let dir = tempdir();
        let subdir = dir.join("folder");
        fs::create_dir(&subdir).unwrap();

        let err = gather_contents(&[subdir]).unwrap_err();
        assert!(err.to_string().contains("not a file"));
    }

    #[test]
    fn errors_on_missing_file() {
        let missing = tempdir().join("missing.txt");
        let err = gather_contents(&[missing]).unwrap_err();
        assert!(err.to_string().contains("No such file"));
    }

    fn tempdir() -> PathBuf {
        let mut dir = env::temp_dir();
        dir.push(format!("pbcat-test-{}", unique_suffix()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn unique_suffix() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{}-{}", std::process::id(), id)
    }

    fn write_file(path: &PathBuf, contents: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())
    }
}
