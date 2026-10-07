// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    process::Command,
};

use regex::Regex;
use walkdir::WalkDir;

/// Executes `git log` once for the entire repository at `repo_path` and returns
/// a mapping from file path to its min and max commit years (as a `RangeInclusive<u32>`).
pub fn get_revision_years(repo_path: impl AsRef<Path>) -> HashMap<PathBuf, RangeInclusive<u32>> {
    let mut file_years = HashMap::new();

    // Run git log once to dump:
    //   <year>
    //   <blank line>
    //   path/to/file1
    //   path/to/file2
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["log", "--name-only", "--format=%ad", "--date=format:%Y"])
        .output();

    let Ok(output) = output else {
        return HashMap::new();
    };

    if !output.status.success() {
        return HashMap::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut current_year: Option<u32> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Check if the line is a 4-digit year header
        if line.len() == 4 && line.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(year) = line.parse::<u32>() {
                current_year = Some(year);
                continue;
            }
        }

        // Path entry associated with the current commit year
        if let Some(year) = current_year {
            let path = PathBuf::from(line);
            file_years
                .entry(path)
                .and_modify(|range: &mut RangeInclusive<u32>| {
                    let min = (*range.start()).min(year);
                    let max = (*range.end()).max(year);
                    *range = min..=max;
                })
                .or_insert(year..=year);
        }
    }

    file_years
}

/// Manages updating copyright notices across source files in a repository.
pub struct CopyrightUpdater {
    path: PathBuf,
    extensions: HashMap<String, String>,
    excluded_patterns: HashSet<PathBuf>,
}

impl CopyrightUpdater {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            extensions: Default::default(),
            excluded_patterns: Default::default(),
        }
    }

    /// Add a single extension and its associated comment prefix (e.g. `"rs"`, `"//"`).
    pub fn extension(mut self, extension: impl Into<String>, prefix: impl Into<String>) -> Self {
        let ext = extension.into();
        self.extensions
            .insert(ext.trim_start_matches('.').to_string(), prefix.into());
        self
    }

    /// Add multiple extensions mapped to the same comment prefix.
    pub fn extensions(mut self, extensions: &[&str], prefix: &str) -> Self {
        for &ext in extensions {
            self = self.extension(ext, prefix);
        }
        self
    }

    /// Add an explicit excluded path or glob pattern.
    pub fn exclude(mut self, path: impl Into<PathBuf>) -> Self {
        self.excluded_patterns.insert(path.into());
        self
    }

    /// Automatically query Git (`git check-ignore`) to collect ignored files/patterns.
    pub fn with_git_exclusions(mut self) -> Self {
        let Ok(output) = Command::new("git")
            .current_dir(&self.path)
            .args(["status", "--ignored", "--porcelain"])
            .output()
        else {
            return self;
        };

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(ignored_path) = line.strip_prefix("!! ") {
                    let trimmed = ignored_path.trim().trim_end_matches('/');
                    self.excluded_patterns.insert(PathBuf::from(trimmed));
                    self.excluded_patterns
                        .insert(PathBuf::from(format!("{trimmed}/*")));
                }
            }
        }
        self
    }
}

impl CopyrightUpdater {
    /// Scan directory, query revision years, and update copyright notices in files.
    pub fn update(&self) -> std::io::Result<()> {
        let revision_years = get_revision_years(&self.path);
        let copyright_re = Regex::new(r"Copyright © (\d{4})(-\d{4})?").unwrap();
        let year = chrono::Datelike::year(&chrono::Utc::now()) as u32;

        // Prepare exclusion glob patterns
        let exclusion_globs: Vec<_> = self
            .excluded_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(&p.to_string_lossy()).ok())
            .collect();

        for path in WalkDir::new(&self.path)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.path().to_path_buf())
            .filter(|path| path.is_file())
        {
            // Relative path check for git matching and exclusion evaluation
            let relative_path = path.strip_prefix(&self.path).unwrap_or(&path);
            let path_str = relative_path.to_string_lossy();

            if exclusion_globs.iter().any(|p| p.matches(&path_str)) {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let Some(prefix) = self.extensions.get(ext) else {
                continue;
            };

            // Determine year range from Git history or default to current year
            let year_range = revision_years
                .get(relative_path)
                .cloned()
                .unwrap_or(year..=year);

            let formatted_years = if year_range.start() == year_range.end() {
                format!("{}", year_range.start())
            } else {
                format!("{}-{}", year_range.start(), year_range.end())
            };

            let content = fs::read_to_string(&path)?;

            let updated_content = if copyright_re.is_match(&content) {
                copyright_re
                    .replace(&content, format!("Copyright © {formatted_years}"))
                    .to_string()
            } else {
                // Header missing: Prepend notice
                let header = format!(
                    "{prefix} Copyright © {formatted_years} The µcad authors <info@microcad.xyz>\n\
                     {prefix} SPDX-License-Identifier: AGPL-3.0-or-later\n\n"
                );
                format!("{header}{content}")
            };

            if updated_content != content {
                println!("cargo:warning=updated: {path:?} -> {formatted_years}");
                fs::write(&path, updated_content.as_bytes())?;
            }
        }

        Ok(())
    }
}
pub fn update_copyright() -> std::io::Result<()> {
    let updater = CopyrightUpdater::new(".")
        .extensions(&["rs", "pest", "slint", "wgsl", "µcad"], "//")
        .extension("toml", "#")
        .exclude("target/*")
        .exclude("crates/*/tests/*.µcad")
        .with_git_exclusions();

    updater.update()
}
