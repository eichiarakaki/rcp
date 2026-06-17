use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::Parser;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "rcp",
    version,
    about = "Copy files and directories recursively to clipboard, merged with path headers"
)]
struct Args {
    #[arg(long, default_value_t = false)]
    copy: bool,

    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,

    #[arg(long, num_args = 0..)]
    exclude_file_types: Vec<String>,

    /// Ignore files/directories/patterns.
    ///
    /// Examples:
    ///   -i target
    ///   -i .git
    ///   -i "*.lock"
    ///   -i "dist/*.js"
    #[arg(short = 'i', long = "ignore", value_name = "PATTERN", action = clap::ArgAction::Append)]
    ignore: Vec<String>,

    /// Disable built-in default ignores.
    #[arg(long, default_value_t = false)]
    no_default_ignores: bool,
}

#[derive(Debug, Clone)]
enum IgnoreRule {
    /// Match any path component by exact name.
    ///
    /// Example:
    ///   target
    ///   .git
    ///   node_modules
    Name(String),

    /// Match by extension including the leading dot.
    ///
    /// Example:
    ///   .lock
    ///   .png
    Extension(String),

    /// Match relative path or file name with a simple '*' wildcard.
    ///
    /// Example:
    ///   result-*
    ///   dist/*.js
    Glob(String),

    /// Match exact absolute path or anything under it.
    Path(PathBuf),
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
            if t.starts_with('.') {
                t.to_lowercase()
            } else {
                format!(".{}", t.to_lowercase())
            }
        })
        .collect();

    let ignore_rules = build_ignore_rules(&cwd, &args.ignore, !args.no_default_ignores);

    let mut files_to_process: Vec<PathBuf> = Vec::new();

    for include_path in &args.paths {
        let abs_include = if include_path.is_absolute() {
            include_path.clone()
        } else {
            cwd.join(include_path)
        };

        let canon_include = fs::canonicalize(&abs_include)
            .ok()
            .unwrap_or_else(|| abs_include.clone());

        if is_path_ignored(&abs_include, &ignore_rules, &cwd)
            || is_path_ignored(&canon_include, &ignore_rules, &cwd)
        {
            continue;
        }

        if abs_include.is_file() {
            if !should_exclude_by_type(&abs_include, &exclude_exts)
                && !is_path_ignored(&abs_include, &ignore_rules, &cwd)
            {
                files_to_process.push(abs_include);
            }
        } else if abs_include.is_dir() {
            let walker = WalkDir::new(&abs_include)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| !is_path_ignored(entry.path(), &ignore_rules, &cwd));

            for entry in walker {
                let entry = entry?;

                if entry.file_type().is_file() {
                    let p = entry.path();

                    if !should_exclude_by_type(p, &exclude_exts)
                        && !is_path_ignored(p, &ignore_rules, &cwd)
                    {
                        files_to_process.push(p.to_path_buf());
                    }
                }
            }
        }
    }

    files_to_process.sort_by(|a, b| {
        display_path(a, &cwd)
            .to_lowercase()
            .cmp(&display_path(b, &cwd).to_lowercase())
    });

    files_to_process.dedup();

    if files_to_process.is_empty() {
        println!("No files matched after applying ignores.");
        return Ok(());
    }

    let mut output = String::with_capacity(128 * 1024 * 1024);

    for file_path in &files_to_process {
        let display_path = display_path(file_path, &cwd);

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

fn default_ignore_patterns() -> Vec<String> {
    [
        // VCS
        ".git",
        ".hg",
        ".svn",

        // Rust / Zig / Go / JVM / C / C++
        "target",
        "zig-cache",
        "zig-out",
        ".zig-cache",
        "result",
        "result-*",
        "build",
        "cmake-build-*",
        ".gradle",
        "out",

        // JS / TS / web
        "node_modules",
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".astro",
        ".vite",
        "dist",
        "coverage",

        // Python
        "__pycache__",
        ".pytest_cache",
        ".mypy_cache",
        ".ruff_cache",
        ".tox",
        ".venv",
        "venv",

        // Nix / direnv / env
        ".direnv",
        ".devenv",

        // Editors / OS trash
        ".DS_Store",
        "Thumbs.db",

        // Locks
        "*.lock",
        "pnpm-lock.yaml",
        "package-lock.json",
        "bun.lockb",
        "bun.lock",

        // Logs / temp
        "*.log",
        "*.tmp",
        "*.temp",
        "*.swp",
        "*.swo",

        // Images
        "*.png",
        "*.jpg",
        "*.jpeg",
        "*.gif",
        "*.webp",
        "*.ico",
        "*.bmp",
        "*.tiff",
        "*.svg",

        // Video / audio
        "*.mp4",
        "*.mov",
        "*.mkv",
        "*.webm",
        "*.mp3",
        "*.wav",
        "*.flac",
        "*.ogg",

        // Archives
        "*.zip",
        "*.tar",
        "*.tar.gz",
        "*.tgz",
        "*.gz",
        "*.xz",
        "*.7z",
        "*.rar",

        // Documents / binaries usually useless for LLM context
        "*.pdf",
        "*.doc",
        "*.docx",
        "*.ppt",
        "*.pptx",
        "*.xls",
        "*.xlsx",

        // Native binaries / objects
        "*.exe",
        "*.dll",
        "*.so",
        "*.dylib",
        "*.o",
        "*.a",
        "*.rlib",
        "*.class",
        "*.jar",
        "*.war",

        // Bytecode / databases / data blobs
        "*.pyc",
        "*.pyo",
        "*.sqlite",
        "*.sqlite3",
        "*.db",
        "*.parquet",
        "*.arrow",
        "*.bin",

        // Fonts
        "*.ttf",
        "*.otf",
        "*.woff",
        "*.woff2",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn build_ignore_rules(cwd: &Path, user_ignores: &[String], include_defaults: bool) -> Vec<IgnoreRule> {
    let mut patterns = Vec::new();

    if include_defaults {
        patterns.extend(default_ignore_patterns());
    }

    patterns.extend(user_ignores.iter().cloned());

    patterns
        .iter()
        .filter_map(|raw| compile_ignore_rule(cwd, raw))
        .collect()
}

fn compile_ignore_rule(cwd: &Path, raw: &str) -> Option<IgnoreRule> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed
        .trim_end_matches('/')
        .replace('\\', "/");

    if normalized.is_empty() {
        return None;
    }

    let lower = normalized.to_lowercase();

    // Extension glob:
    //   *.lock
    //   *.png
    if lower.starts_with("*.") && !lower[2..].contains('*') && !lower.contains('/') {
        return Some(IgnoreRule::Extension(format!(".{}", &lower[2..])));
    }

    // General glob:
    //   result-*
    //   cmake-build-*
    //   dist/*.js
    if lower.contains('*') {
        return Some(IgnoreRule::Glob(lower));
    }

    // Path-like ignore:
    //   src/generated
    //   ./dist
    //   /absolute/path
    if lower.contains('/') || Path::new(&normalized).is_absolute() {
        let p = PathBuf::from(&normalized);

        let abs = if p.is_absolute() {
            p
        } else {
            cwd.join(p)
        };

        let canon = fs::canonicalize(&abs).ok().unwrap_or(abs);

        return Some(IgnoreRule::Path(canon));
    }

    // Plain name:
    //   .git
    //   target
    //   node_modules
    Some(IgnoreRule::Name(lower))
}

fn is_path_ignored(p: &Path, rules: &[IgnoreRule], cwd: &Path) -> bool {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    };

    let canon = fs::canonicalize(&abs).ok();

    let rel = abs.strip_prefix(cwd).unwrap_or(&abs);
    let rel_s = path_to_slash(rel).to_lowercase();

    let file_name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let components: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect();

    for rule in rules {
        match rule {
            IgnoreRule::Name(name) => {
                if components.iter().any(|c| c == name) || file_name == *name {
                    return true;
                }
            }

            IgnoreRule::Extension(ext) => {
                if let Some(path_ext) = p.extension() {
                    let ext_with_dot = format!(".{}", path_ext.to_string_lossy().to_lowercase());
                    if ext_with_dot == *ext {
                        return true;
                    }
                }
            }

            IgnoreRule::Glob(pattern) => {
                if wildcard_match(pattern, &rel_s) || wildcard_match(pattern, &file_name) {
                    return true;
                }
            }

            IgnoreRule::Path(ignore_path) => {
                if abs == *ignore_path || abs.starts_with(ignore_path) {
                    return true;
                }

                if let Some(canon_path) = &canon {
                    if canon_path == ignore_path || canon_path.starts_with(ignore_path) {
                        return true;
                    }
                }
            }
        }
    }

    false
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

fn display_path(file_path: &Path, cwd: &Path) -> String {
    file_path
        .strip_prefix(cwd)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.display().to_string())
}

fn path_to_slash(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();

    let mut p = 0;
    let mut t = 0;

    let mut star: Option<usize> = None;
    let mut match_after_star = 0;

    while t < text.len() {
        if p < pattern.len() && pattern[p] == text[t] {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            match_after_star = t;
            p += 1;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            match_after_star += 1;
            t = match_after_star;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}

/// Copies text to clipboard.
/// Prefers wl-copy on Wayland. Falls back to xclip/xsel on X11.
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Wayland
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

    Err("Failed to copy to clipboard. Install 'wl-copy' on Wayland or 'xclip'/'xsel' on X11.".into())
}
