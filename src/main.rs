use ignore::{DirEntry, WalkBuilder};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            if summary.listed {
                println!(
                    "Listed {} file{} ({} bytes)",
                    summary.files,
                    if summary.files == 1 { "" } else { "s" },
                    summary.bytes
                );
            } else {
                println!(
                    "Copied {} file{} ({} bytes) to clipboard",
                    summary.files,
                    if summary.files == 1 { "" } else { "s" },
                    summary.bytes
                );
            }
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
    listed: bool,
}

struct Options {
    separator: Option<String>,
    header: bool,
    sort: SortMode,
    list_only: bool,
    inputs: Vec<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Args,
    Name,
}

const SKIP_DIR_NAMES: &[&str] = &[
    "DerivedData",
    "Pods",
    "target",
    "node_modules",
    "build",
    ".build",
    ".gradle",
    "Carthage",
    "buck-out",
    "__pycache__",
    ".idea",
    ".git",
];

fn run() -> Result<CopySummary, Box<dyn Error>> {
    let options = parse_args()?;

    let files = collect_files(&options.inputs)?;
    let files = order_files(files, options.sort);
    if files.is_empty() {
        return Err("No files to copy".into());
    }

    if options.list_only {
        let bytes = total_bytes(&files)?;
        for path in &files {
            println!("{}", display(path));
        }
        return Ok(CopySummary {
            files: files.len(),
            bytes,
            listed: true,
        });
    }

    let bytes = copy_files_to_clipboard(&files, options.separator.as_deref(), options.header)?;

    Ok(CopySummary {
        files: files.len(),
        bytes,
        listed: false,
    })
}

fn parse_args() -> Result<Options, Box<dyn Error>> {
    let mut inputs = Vec::new();
    let mut separator: Option<String> = None;
    let mut header = false;
    let mut sort = SortMode::Args;
    let mut list_only = false;
    let mut args = env::args_os().skip(1).peekable();
    let mut end_of_options = false;

    while let Some(arg) = args.next() {
        if !end_of_options {
            if arg == "--" {
                end_of_options = true;
                continue;
            }
            if arg == "-s" || arg == "--separator" {
                let value = args
                    .next()
                    .ok_or("Missing value for separator (-s/--separator)")?;
                separator = Some(
                    value
                        .into_string()
                        .map_err(|_| "Separator must be valid UTF-8")?,
                );
                continue;
            }
            if arg == "-H" || arg == "--header" {
                header = true;
                continue;
            }
            if arg == "--sort" {
                let value = args.next().ok_or("Missing value for --sort")?;
                sort = parse_sort(&value)?;
                continue;
            }
            if arg == "-L" || arg == "--list" {
                list_only = true;
                continue;
            }
            if arg.to_string_lossy().starts_with('-') && arg != "-" {
                return Err(format!("Unknown option: {}", arg.to_string_lossy()).into());
            }
        }
        inputs.push(PathBuf::from(arg));
    }

    if inputs.is_empty() {
        return Err(
            "Usage: pbcat [-s <separator>] [-H|--header] [--sort name|args] [-L|--list] <file|dir> [more ...]"
                .into(),
        );
    }

    Ok(Options {
        separator,
        header,
        sort,
        list_only,
        inputs,
    })
}

fn parse_sort(value: &std::ffi::OsStr) -> Result<SortMode, Box<dyn Error>> {
    let value = value
        .to_str()
        .ok_or("Sort value must be valid UTF-8 (args|name)")?;
    match value {
        "args" => Ok(SortMode::Args),
        "name" => Ok(SortMode::Name),
        _ => Err("Sort must be one of: args, name".into()),
    }
}

fn collect_files(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    for input in inputs {
        let metadata = fs::metadata(input).map_err(|e| format!("{}: {}", display(input), e))?;
        if metadata.is_file() {
            add_file(input, &mut files, &mut seen)?;
            continue;
        }
        if metadata.is_dir() {
            collect_dir(input, &mut files, &mut seen)?;
            continue;
        }
        return Err(format!("{}: not a file or directory", display(input)).into());
    }

    Ok(files)
}

fn order_files(mut files: Vec<PathBuf>, sort: SortMode) -> Vec<PathBuf> {
    match sort {
        SortMode::Args => files,
        SortMode::Name => {
            files.sort();
            files
        }
    }
}

fn total_bytes(paths: &[PathBuf]) -> Result<usize, Box<dyn Error>> {
    let mut total: usize = 0;
    for path in paths {
        let meta = fs::metadata(path).map_err(|e| format!("{}: {}", display(path), e))?;
        let len = meta.len();
        let add = usize::try_from(len).unwrap_or_else(|_| usize::MAX.saturating_sub(total));
        total = total.saturating_add(add);
    }
    Ok(total)
}

fn collect_dir(
    dir: &PathBuf,
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let walker = WalkBuilder::new(dir)
        .standard_filters(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .add_custom_ignore_filename(".gitignore")
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                !should_skip_dir(entry)
            } else {
                true
            }
        })
        .sort_by_file_name(|a, b| a.cmp(b))
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            add_file(&entry.into_path(), files, seen)?;
        }
    }

    Ok(())
}

fn add_file(
    path: &PathBuf,
    files: &mut Vec<PathBuf>,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let canonical = fs::canonicalize(path).map_err(|e| format!("{}: {}", display(path), e))?;
    if seen.insert(canonical.clone()) {
        files.push(canonical);
    }
    Ok(())
}

fn should_skip_dir(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|name| SKIP_DIR_NAMES.contains(&name))
        .unwrap_or(false)
}

#[cfg(test)]
fn gather_contents(
    paths: &[PathBuf],
    separator: Option<&str>,
    header: bool,
) -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();

    for (idx, path) in paths.iter().enumerate() {
        let metadata = fs::metadata(path).map_err(|e| format!("{}: {}", display(path), e))?;
        if !metadata.is_file() {
            return Err(format!("{}: not a file", display(path)).into());
        }

        let bytes = fs::read(path).map_err(|e| format!("{}: {}", display(path), e))?;
        let text =
            String::from_utf8(bytes).map_err(|_| format!("{}: not valid UTF-8", display(path)))?;
        if header {
            buffer.push_str(&format!("== {} ==\n", display(path)));
        }
        buffer.push_str(&text);
        if let Some(sep) = separator && idx + 1 < paths.len() {
            buffer.push_str(sep);
        }
    }

    Ok(buffer)
}

fn write_files<W: Write>(
    paths: &[PathBuf],
    separator: Option<&str>,
    header: bool,
    mut out: W,
) -> Result<usize, Box<dyn Error>> {
    let mut total = 0usize;

    for (idx, path) in paths.iter().enumerate() {
        let bytes = fs::read_to_string(path).map_err(|e| format!("{}: {}", display(path), e))?;
        if header {
            let header_text = format!("== {} ==\n", display(path));
            total += header_text.len();
            out.write_all(header_text.as_bytes())?;
        }
        total += bytes.len();
        out.write_all(bytes.as_bytes())?;

        if let Some(sep) = separator && idx + 1 < paths.len() {
            total += sep.len();
            out.write_all(sep.as_bytes())?;
        }
    }

    Ok(total)
}

fn copy_files_to_clipboard(
    files: &[PathBuf],
    separator: Option<&str>,
    header: bool,
) -> Result<usize, Box<dyn Error>> {
    if let Ok(path) = env::var("PBCAT_CLIPBOARD_FILE") {
        let file = fs::File::create(path)?;
        return write_files(files, separator, header, file);
    }

    let mut last_err: Option<String> = None;
    for tool in preferred_clipboard_tools() {
        match Command::new(tool.program)
            .args(tool.args)
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    let write_result = write_files(files, separator, header, &mut stdin);
                    drop(stdin);
                    let status = child.wait()?;
                    if status.success() && write_result.is_ok() {
                        return write_result;
                    }
                    last_err = Some(format!("{} exited with status {}", tool.program, status));
                } else {
                    last_err = Some("failed to open stdin".into());
                }
            }
            Err(e) => {
                last_err = Some(format!("{}: {}", tool.program, e));
            }
        }
    }

    Err(last_err
        .unwrap_or_else(|| {
            "No supported clipboard utility found (tried pbcopy/wl-copy/xclip/xsel)".to_string()
        })
        .into())
}

struct ClipboardTool {
    program: &'static str,
    args: &'static [&'static str],
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

#[cfg(target_os = "windows")]
fn preferred_clipboard_tools() -> Vec<ClipboardTool> {
    vec![
        ClipboardTool {
            program: "clip",
            args: &[],
        },
        ClipboardTool {
            program: "powershell",
            args: &["-NoProfile", "-Command", "Set-Clipboard"],
        },
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn preferred_clipboard_tools() -> Vec<ClipboardTool> {
    Vec::new()
}

fn display(path: &std::path::Path) -> String {
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

        let combined = gather_contents(&[first, second], None, false).unwrap();
        assert_eq!(combined, "helloworld");
    }

    #[test]
    fn rejects_non_files() {
        let dir = tempdir();
        let subdir = dir.join("folder");
        fs::create_dir(&subdir).unwrap();

        let err = gather_contents(&[subdir], None, false).unwrap_err();
        assert!(err.to_string().contains("not a file"));
    }

    #[test]
    fn errors_on_missing_file() {
        let missing = tempdir().join("missing.txt");
        let err = gather_contents(&[missing], None, false).unwrap_err();
        assert!(err.to_string().contains("No such file"));
    }

    #[test]
    fn collects_directory_recursively_respecting_gitignore() {
        let dir = tempdir();
        let ignored = dir.join("skip.log");
        let keep = dir.join("keep.txt");
        let nested_dir = dir.join("nested");
        let nested_keep = nested_dir.join("inner.txt");

        write_file(&keep, "keep").unwrap();
        fs::create_dir_all(&nested_dir).unwrap();
        write_file(&nested_keep, "inner").unwrap();
        write_file(&ignored, "skip").unwrap();
        write_file(&dir.join(".gitignore"), "*.log\n").unwrap();

        let files = collect_files(std::slice::from_ref(&dir)).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();

        assert_eq!(names, vec!["keep.txt", "inner.txt"]);
    }

    #[test]
    fn dedupes_files_seen_via_args_and_directory_walk() {
        let dir = tempdir();
        let file = dir.join("one.txt");
        write_file(&file, "once").unwrap();

        let files = collect_files(&[file.clone(), dir]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap().to_string_lossy(), "one.txt");
    }

    #[test]
    fn inserts_separator_between_files() {
        let dir = tempdir();
        let first = dir.join("a.txt");
        let second = dir.join("b.txt");

        write_file(&first, "alpha").unwrap();
        write_file(&second, "beta").unwrap();

        let combined = gather_contents(&[first, second], Some("\n---\n"), false).unwrap();
        assert_eq!(combined, "alpha\n---\nbeta");
    }

    #[test]
    fn adds_headers_when_enabled() {
        let dir = tempdir();
        let file = dir.join("file.txt");
        write_file(&file, "body").unwrap();

        let combined = gather_contents(std::slice::from_ref(&file), None, true).unwrap();
        let text = combined.replace(display(&file).as_str(), "PATH");
        assert_eq!(text, "== PATH ==\nbody");
    }

    #[test]
    fn sorts_by_name_when_requested() {
        let dir = tempdir();
        let b = dir.join("b.txt");
        let a = dir.join("a.txt");
        write_file(&a, "a").unwrap();
        write_file(&b, "b").unwrap();

        let files = order_files(vec![b.clone(), a.clone()], SortMode::Name);
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt"]);
    }

    #[test]
    fn skips_known_build_dirs() {
        let dir = tempdir();
        let derived = dir.join("DerivedData");
        let nested = derived.join("output.txt");
        let keep = dir.join("keep.txt");

        fs::create_dir_all(derived).unwrap();
        write_file(&nested, "should_skip").unwrap();
        write_file(&keep, "keep").unwrap();

        let files = collect_files(&[dir]).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();

        assert_eq!(names, vec!["keep.txt"]);
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
