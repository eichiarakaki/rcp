use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::fs;
use std::env;
use std::process::{Command, Stdio};
use std::io::Write;
use clap::Parser;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "rcp", version, about = "Copy files and directories recursively to clipboard (merged with path headers)")]
struct Args {
    #[arg(long, default_value_t = false)]
    copy: bool,

    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,

    #[arg(long, num_args = 0..)]
    exclude_file_types: Vec<String>,

    #[arg(long, num_args = 0..)]
    exclude: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.paths.is_empty() {
        eprintln!("Error: At least one path is required.");
        std::process::exit(1);
    }

    let cwd = env::current_dir()?;

    let exclude_exts: HashSet<String> = args
        .exclude_file_types
        .iter()
        .map(|s| {
            let t = s.trim();
            if t.starts_with('.') { t.to_lowercase() } else { format!(".{}", t.to_lowercase()) }
        })
        .collect();

    let exclude_set: HashSet<PathBuf> = args
        .exclude
        .iter()
        .filter_map(|p| {
            let abs = if p.is_absolute() { p.clone() } else { cwd.join(p) };
            fs::canonicalize(&abs).ok().or(Some(abs))
        })
        .collect();

    let mut files_to_process: Vec<PathBuf> = Vec::new();

    for include_path in &args.paths {
        let abs_include = if include_path.is_absolute() {
            include_path.clone()
        } else {
            cwd.join(include_path)
        };

        let canon_include = fs::canonicalize(&abs_include).ok().unwrap_or_else(|| abs_include.clone());

        if exclude_set.contains(&abs_include) || exclude_set.contains(&canon_include) {
            continue;
        }

        if include_path.is_file() || abs_include.is_file() {
            if !should_exclude_by_type(include_path, &exclude_exts) {
                files_to_process.push(include_path.clone());
            }
        } else if include_path.is_dir() || abs_include.is_dir() {
            let walker = WalkDir::new(include_path)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| !is_path_excluded(entry.path(), &exclude_set, &cwd));

            for entry in walker {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let p = entry.path();
                    if !should_exclude_by_type(p, &exclude_exts) {
                        files_to_process.push(p.to_path_buf());
                    }
                }
            }
        }
    }

    files_to_process.sort_by(|a, b| {
        a.to_string_lossy().to_lowercase().cmp(&b.to_string_lossy().to_lowercase())
    });
    files_to_process.dedup();

    if files_to_process.is_empty() {
        println!("No files matched after applying excludes.");
        return Ok(());
    }

    // Large capacity for big projects
    let mut output = String::with_capacity(128 * 1024 * 1024);

    for file_path in &files_to_process {
        let display_path = file_path
            .strip_prefix(&cwd)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_path.display().to_string());

        let content = match fs::read(file_path) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
            Err(e) => format!("<Error reading file: {}>", e),
        };

        output.push_str(&format!("{}:\n", display_path));
        output.push_str(&content);
        if !content.ends_with('\n') {
            output.push('\n');
        }
        output.push('\n');
    }

    copy_to_clipboard(&output)?;

    let file_count = files_to_process.len();
    let char_count = output.len();

    println!(
        "Copied {} file(s) to clipboard ({} characters, ~{:.1} MiB)",
        file_count,
        char_count,
        char_count as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}

fn should_exclude_by_type(p: &Path, exclude_exts: &HashSet<String>) -> bool {
    if exclude_exts.is_empty() {
        return false;
    }
    if let Some(ext) = p.extension() {
        let ext_with_dot = format!(".{}", ext.to_string_lossy().to_lowercase());
        exclude_exts.contains(&ext_with_dot)
    } else {
        false
    }
}

fn is_path_excluded(p: &Path, exclude_set: &HashSet<PathBuf>, cwd: &Path) -> bool {
    let abs = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    if exclude_set.contains(&abs) {
        return true;
    }
    if let Ok(canon) = fs::canonicalize(&abs) {
        return exclude_set.contains(&canon);
    }
    false
}

/// Copies text to clipboard.
/// Prefers wl-copy (stdin) on Wayland. Falls back to xclip/xsel on X11.
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Try wl-copy first using stdin (robust for large content)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if let Ok(status) = child.wait() {
                    if status.success() {
                        return Ok(());
                    }
                }
            }
        }
    }

    // X11 fallback
    if env::var("DISPLAY").is_ok() {
        // xclip
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                return Ok(());
            }
        }

        // xsel
        if let Ok(mut child) = Command::new("xsel")
            .args(["--clipboard", "--input"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            if child.wait().map(|s| s.success()).unwrap_or(false) {
                return Ok(());
            }
        }
    }

    Err("Failed to copy to clipboard. Install 'wl-copy' (Wayland) or 'xclip'/'xsel' (X11).".into())
}
