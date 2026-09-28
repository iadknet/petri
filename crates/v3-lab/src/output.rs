//! Run directory under the calling checkout's ignored `.bench-artifacts/`,
//! with a byte cap checked before every write. Nothing here is committed.

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::LabError;

/// Bytes set aside for the summary, before overlay bytes.
pub const SUMMARY_RESERVE: u64 = 1 << 20;
/// Default per-run-directory cap.
pub const DEFAULT_BYTE_CAP: u64 = 64 << 20;

/// The byte budget: rows and elites may use `cap − reserve`; the summary
/// must fit in `reserve`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    cap: u64,
    reserve: u64,
    used: u64,
}

impl Budget {
    /// # Errors
    ///
    /// [`LabError::Output`] when `cap` is below twice `reserve`.
    pub fn new(cap: u64, reserve: u64) -> Result<Self, LabError> {
        if cap < reserve.saturating_mul(2) {
            return Err(LabError::Output(format!(
                "byte cap {cap} is below twice the summary reserve {reserve}"
            )));
        }
        Ok(Self {
            cap,
            reserve,
            used: 0,
        })
    }

    /// Whether `bytes` more fit the rows-and-elites budget; counts them if so.
    pub fn admit(&mut self, bytes: u64) -> bool {
        if self.used + bytes > self.cap - self.reserve {
            return false;
        }
        self.used += bytes;
        true
    }

    #[must_use]
    pub fn reserve(&self) -> u64 {
        self.reserve
    }
}

/// An open run directory.
#[derive(Debug)]
pub struct RunDir {
    pub path: PathBuf,
    budget: Budget,
    rows: File,
}

fn dir_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        total += if meta.is_dir() {
            dir_size(&entry.path())?
        } else {
            meta.len()
        };
    }
    Ok(total)
}

impl RunDir {
    /// Create `path` (inside the checkout's `.bench-artifacts/`) and open
    /// `rows.ndjson`; existing bytes in the directory count against `budget`.
    ///
    /// # Errors
    ///
    /// I/O failures, or [`LabError::Output`] if existing content already
    /// exceeds the budget.
    pub fn create(path: PathBuf, mut budget: Budget) -> Result<Self, LabError> {
        fs::create_dir_all(path.join("elites"))?;
        if !budget.admit(dir_size(&path)?) {
            return Err(LabError::Output(format!(
                "{} already exceeds the byte cap",
                path.display()
            )));
        }
        let rows = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.join("rows.ndjson"))?;
        Ok(Self { path, budget, rows })
    }

    /// Append one NDJSON line. `Ok(false)` means the cap refused it and
    /// nothing was written.
    ///
    /// # Errors
    ///
    /// I/O failures.
    pub fn write_row(&mut self, line: &str) -> Result<bool, LabError> {
        let bytes = line.len() as u64 + 1;
        if !self.budget.admit(bytes) {
            return Ok(false);
        }
        self.rows.write_all(line.as_bytes())?;
        self.rows.write_all(b"\n")?;
        Ok(true)
    }

    /// Write `elites/<name>.json`. `Ok(false)` means the cap refused it.
    ///
    /// # Errors
    ///
    /// I/O failures.
    pub fn write_elite(&mut self, name: &str, content: &[u8]) -> Result<bool, LabError> {
        if !self.budget.admit(content.len() as u64) {
            return Ok(false);
        }
        fs::write(
            self.path.join("elites").join(format!("{name}.json")),
            content,
        )?;
        Ok(true)
    }

    /// Write `summary.json` inside the reserve.
    ///
    /// # Errors
    ///
    /// [`LabError::Output`] when the summary exceeds the reserve; I/O
    /// failures.
    pub fn write_summary(&mut self, content: &[u8]) -> Result<(), LabError> {
        if content.len() as u64 > self.budget.reserve() {
            return Err(LabError::Output(format!(
                "summary of {} bytes exceeds the {}-byte reserve",
                content.len(),
                self.budget.reserve()
            )));
        }
        self.rows.flush()?;
        fs::write(self.path.join("summary.json"), content)?;
        Ok(())
    }
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The calling checkout's root (`git rev-parse --show-toplevel`).
///
/// # Errors
///
/// [`LabError::Io`] outside a git checkout.
pub fn checkout_root() -> Result<PathBuf, LabError> {
    git(&["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .ok_or_else(|| LabError::Io("not inside a git checkout".into()))
}

/// `(git_revision, dirty)` of the calling checkout, `None` when unknown.
#[must_use]
pub fn git_state() -> (Option<String>, Option<bool>) {
    (
        git(&["rev-parse", "HEAD"]),
        git(&["status", "--porcelain"]).map(|status| !status.is_empty()),
    )
}

/// Resolve the run directory: `out` when given, which must resolve inside
/// `<root>/.bench-artifacts/`, else `<root>/.bench-artifacts/lab/<default_name>`.
///
/// # Errors
///
/// [`LabError::Output`] for a path outside `.bench-artifacts/`.
pub fn resolve_out(
    root: &Path,
    out: Option<&Path>,
    default_name: &str,
) -> Result<PathBuf, LabError> {
    let artifacts = root.join(".bench-artifacts");
    fs::create_dir_all(&artifacts)?;
    let artifacts = artifacts.canonicalize()?;
    let Some(out) = out else {
        return Ok(artifacts.join("lab").join(default_name));
    };
    let absolute = if out.is_absolute() {
        out.to_path_buf()
    } else {
        std::env::current_dir()?.join(out)
    };
    // Canonicalize the deepest existing ancestor, then re-append the rest.
    let mut existing = absolute.as_path();
    let mut rest = Vec::new();
    while !existing.exists() {
        rest.push(existing.file_name().ok_or_else(|| {
            LabError::Output(format!("--out {} has no existing ancestor", out.display()))
        })?);
        existing = existing.parent().unwrap_or(Path::new("/"));
    }
    let mut resolved = existing.canonicalize()?;
    for part in rest.iter().rev() {
        if *part == ".." || *part == "." {
            return Err(LabError::Output(
                "--out may not contain `.` or `..` components".into(),
            ));
        }
        resolved.push(part);
    }
    if resolved == artifacts || !resolved.starts_with(&artifacts) {
        return Err(LabError::Output(format!(
            "--out {} must resolve inside {}",
            out.display(),
            artifacts.display()
        )));
    }
    Ok(resolved)
}

/// UTC timestamp `YYYYMMDDTHHMMSSZ` for run directory names.
#[must_use]
pub fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rem = secs % 86_400;
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}T{:02}{:02}{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_refuses_caps_below_twice_the_reserve_and_stops_at_cap_minus_reserve() {
        assert!(Budget::new(199, 100).is_err());
        let mut budget = Budget::new(300, 100).unwrap();
        assert!(budget.admit(150));
        assert!(budget.admit(50));
        assert!(!budget.admit(1));
    }

    #[test]
    fn out_must_stay_inside_bench_artifacts() {
        let root = checkout_root().unwrap();
        assert!(resolve_out(&root, Some(&root.join("docs/lab")), "x").is_err());
        assert!(resolve_out(&root, Some(&root.join(".bench-artifacts")), "x").is_err());
        assert!(resolve_out(
            &root,
            Some(&root.join(".bench-artifacts/lab/../../docs")),
            "x"
        )
        .is_err());
        let inside = resolve_out(&root, Some(&root.join(".bench-artifacts/lab/t")), "x").unwrap();
        assert!(inside.ends_with(".bench-artifacts/lab/t"));
        let default = resolve_out(&root, None, "food-seeking-1-x").unwrap();
        assert!(default.ends_with(".bench-artifacts/lab/food-seeking-1-x"));
    }

    #[test]
    fn utc_stamp_has_the_directory_shape() {
        let stamp = utc_stamp();
        assert_eq!(stamp.len(), 16);
        assert!(stamp.starts_with("20") && stamp.ends_with('Z'));
    }
}
