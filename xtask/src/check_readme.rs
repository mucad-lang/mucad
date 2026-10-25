// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Check the shell commands of `README.md` files one by one manually.

use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};

use mucad_driver::prelude as mu;

/// The languages of code blocks with shell commands.
const SHELL_LANGUAGES: &[&str] = &["sh", "bash", "shell", "zsh", "console"];

/// Directories, which are not searched for `README.md` files, e.g. build outputs like the
/// `pkg` of `wasm-pack`, which contains a copy of the README of its crate.
const EXCLUDED_DIRS: &[&str] = &["target", "node_modules", "dist", "pkg", ".git"];

/// A shell command of a `README.md` file.
#[derive(Debug, PartialEq)]
pub struct Command {
    /// The `README.md` file.
    pub file: PathBuf,
    /// The line of the command in the file, starting at 1.
    pub line: usize,
    /// The command, continued lines are joined.
    pub command: String,
    /// The directory to run the command in, relative to the repository root, e.g. after
    /// `cd editors/web-viewer` in the same code block.
    pub dir: PathBuf,
}

/// The result of checking a command.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Check {
    Ok,
    Failed,
    Skipped,
}

/// The `README.md` files in `paths`, which are files or directories to search.
pub fn readme_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files: Vec<_> = paths
        .iter()
        .flat_map(|path| {
            walkdir::WalkDir::new(path)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|entry| {
                    entry.depth() == 0
                        || !EXCLUDED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref())
                })
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file() && entry.file_name() == "README.md")
                .map(|entry| entry.into_path())
        })
        .collect();
    files.dedup();
    files
}

/// The shell commands in the code blocks of the markdown `text` of `file`, also in lists.
///
/// Fails, if the markdown is invalid, e.g. with an unclosed code block.
pub fn commands(file: &Path, text: &str) -> Result<Vec<Command>, mu::md::ParseError> {
    let md: mu::md::Markdown = text.parse()?;
    Ok(md
        .code_blocks()
        .filter(|block| {
            block
                .header
                .lang
                .as_deref()
                .is_some_and(|lang| SHELL_LANGUAGES.contains(&lang))
        })
        .flat_map(|block| block_commands(file, block))
        .collect())
}

/// The commands of a shell code block of `file`.
///
/// Commands are written for the repository root, a `cd` changes the directory of the following
/// commands of the code block.
fn block_commands(file: &Path, block: &mu::md::CodeBlock) -> Vec<Command> {
    // A console block contains commands after a prompt, and their output.
    let console = block.header.lang.as_deref() == Some("console");
    let mut commands = Vec::new();
    // The directory of the following commands.
    let mut dir = PathBuf::from(".");
    // A command continued with `\` on the next line, with its first line.
    let mut continued: Option<(usize, String)> = None;

    for (index, line) in block.code().lines().enumerate() {
        let trimmed = line.trim();
        let part = match &continued {
            Some(_) => trimmed,
            None if console => match trimmed.strip_prefix("$ ") {
                Some(command) => command,
                None => continue,
            },
            None => trimmed.strip_prefix("$ ").unwrap_or(trimmed),
        };
        if continued.is_none() && (part.is_empty() || part.starts_with('#')) {
            continue;
        }

        // The first line of the code is the line after the line offset.
        let line_no = block.line_offset + index + 1;
        let (line_no, mut command) = continued.take().unwrap_or((line_no, String::new()));
        match part.strip_suffix('\\') {
            Some(start) => {
                command.push_str(start.trim_end());
                command.push(' ');
                continued = Some((line_no, command));
            }
            None => {
                command.push_str(part);
                let command = command.trim();
                // A plain `cd` changes the directory instead of being run, e.g. `cd editors/vscode`.
                match change_dir(command) {
                    Some(path) => dir = dir.join(path),
                    None => commands.push(Command {
                        file: file.to_path_buf(),
                        line: line_no,
                        command: command.to_string(),
                        dir: dir.clone(),
                    }),
                }
            }
        }
    }
    commands
}

/// The directory of a plain `cd` command, e.g. `editors/vscode` for `cd editors/vscode`,
/// `None` for other commands, e.g. `cd x && npm install`, which are run by the shell.
fn change_dir(command: &str) -> Option<&str> {
    let path = command.strip_prefix("cd ")?.trim();
    let is_plain = !path.is_empty()
        && !path.contains(|c: char| "&;|<>$`#'\"".contains(c) || c.is_whitespace());
    is_plain.then_some(path)
}

/// Ask a question on stdin, returns the answer in lower case or `default` for an empty answer.
fn ask(question: &str, default: &str) -> std::io::Result<String> {
    print!("{question} ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer)? == 0 {
        // No more input, e.g. at the end of a pipe.
        return Ok("q".into());
    }
    let answer = answer.trim().to_lowercase();
    Ok(if answer.is_empty() {
        default.to_string()
    } else {
        answer
    })
}

/// Check the commands of the `README.md` files in `paths` one by one: each command can be run,
/// skipped or the check quit, after running it the user decides if it worked.
///
/// With `list`, the commands are only listed.
pub fn check_readme(paths: &[PathBuf], list: bool) -> Result<(), Box<dyn Error>> {
    let mut all = Vec::new();
    for file in readme_files(paths) {
        let text = std::fs::read_to_string(&file)?;
        match commands(&file, &text) {
            Ok(commands) => all.extend(commands),
            // The other files are still checked.
            Err(err) => eprintln!("⚠️  {} is skipped: {err}", file.display()),
        }
    }

    if list {
        for command in &all {
            // The directory, if the command does not run in the repository root.
            let dir = match command.dir.as_os_str() == "." {
                true => String::new(),
                false => format!(" (in {})", command.dir.display()),
            };
            println!(
                "{}:{}:{dir} {}",
                command.file.display(),
                command.line,
                command.command
            );
        }
        return Ok(());
    }

    let mut checks = Vec::new();
    let mut quit = false;
    for (index, command) in all.iter().enumerate() {
        println!(
            "\n[{}/{}] {}:{} (in {})\n  $ {}",
            index + 1,
            all.len(),
            command.file.display(),
            command.line,
            command.dir.display(),
            command.command
        );
        let check = loop {
            match ask("Run? [r]un, [s]kip, [q]uit (r):", "r")?.as_str() {
                "r" | "run" => {
                    let status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&command.command)
                        .current_dir(&command.dir)
                        .status()?;
                    // E.g. `exit status: 1`.
                    println!("{status}");
                    let default = if status.success() { "y" } else { "n" };
                    let question = format!("Did it work? [y]es, [n]o ({default}):");
                    break match ask(&question, default)?.as_str() {
                        "y" | "yes" => Check::Ok,
                        _ => Check::Failed,
                    };
                }
                "s" | "skip" => break Check::Skipped,
                "q" | "quit" => {
                    quit = true;
                    break Check::Skipped;
                }
                _ => println!("Please answer r, s or q."),
            }
        };
        checks.push(check);
        if quit {
            break;
        }
    }
    // After quitting, the remaining commands are skipped.
    checks.resize(all.len(), Check::Skipped);

    summary(&all, &checks)
}

/// Print the result of each command, fails if a command failed.
fn summary(commands: &[Command], checks: &[Check]) -> Result<(), Box<dyn Error>> {
    println!("\nSummary:");
    for (command, check) in commands.iter().zip(checks) {
        let mark = match check {
            Check::Ok => "✅ ok     ",
            Check::Failed => "❌ failed ",
            Check::Skipped => "⏭️ skipped",
        };
        println!(
            "{mark} {}:{}: {}",
            command.file.display(),
            command.line,
            command.command
        );
    }
    let failed = checks
        .iter()
        .filter(|check| **check == Check::Failed)
        .count();
    match failed {
        0 => Ok(()),
        _ => Err(format!("{failed} README command(s) failed").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_texts(text: &str) -> Vec<(usize, String)> {
        commands(Path::new("README.md"), text)
            .expect("Valid markdown")
            .into_iter()
            .map(|command| (command.line, command.command))
            .collect()
    }

    /// Commands of shell code blocks, without comments, prompts and code of other languages.
    #[test]
    fn shell_commands() {
        let text = "\
# Title

```sh
# Build
cargo build

$ cargo test
```

```rust
fn main() {}
```

```bash
npm run wasm \\
    --release
```

```console
$ mucad --version
mucad 0.6.0
```
";
        assert_eq!(
            command_texts(text),
            [
                (5, "cargo build".to_string()),
                (7, "cargo test".to_string()),
                (15, "npm run wasm --release".to_string()),
                (20, "mucad --version".to_string()),
            ]
        );
    }

    /// A plain `cd` changes the directory of the following commands of its code block.
    #[test]
    fn change_directory() {
        let text = "\
```sh
cd editors/web-viewer
npm install
cd x && npm test
```

```sh
cargo test
```
";
        let commands: Vec<_> = commands(Path::new("README.md"), text)
            .expect("Valid markdown")
            .into_iter()
            .map(|command| (command.command, command.dir))
            .collect();
        assert_eq!(
            commands,
            [
                (
                    "npm install".to_string(),
                    PathBuf::from("./editors/web-viewer")
                ),
                (
                    "cd x && npm test".to_string(),
                    PathBuf::from("./editors/web-viewer")
                ),
                ("cargo test".to_string(), PathBuf::from(".")),
            ]
        );
    }

    /// Commands in list items are found, invalid markdown fails.
    #[test]
    fn list_items_and_errors() {
        let text = "\
# Build

1. Install the dependencies:

   ```sh
   npm install
   ```
";
        assert_eq!(command_texts(text), [(6, "npm install".to_string())]);

        assert!(commands(Path::new("README.md"), "```sh\nnpm install\n").is_err());
    }
}
