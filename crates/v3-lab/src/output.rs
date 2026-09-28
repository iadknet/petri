//! Run directory under the injected root's ignored `.bench-artifacts/`,
//! with a byte cap checked before every write. Nothing here is committed.

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::summary::GitMarker;
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

const ROWS: &str = "rows.ndjson";
const SUMMARY: &str = "summary.json";
const ELITES: &str = "elites";
/// What a run writes; a directory holding any of these is not reused.
const RUN_OUTPUTS: [&str; 3] = [ROWS, SUMMARY, ELITES];

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
    /// Create `path` (inside the root's `.bench-artifacts/`) and open
    /// `rows.ndjson`; existing bytes in the directory count against `budget`.
    ///
    /// # Errors
    ///
    /// I/O failures, or [`LabError::Output`] if `path` already holds a
    /// run's outputs (checked before anything is written) or existing
    /// content already exceeds the budget.
    pub fn create(path: PathBuf, mut budget: Budget) -> Result<Self, LabError> {
        if let Some(prior) = RUN_OUTPUTS.iter().find(|name| path.join(name).exists()) {
            return Err(LabError::Output(format!(
                "{} already holds a run's {prior}; choose a fresh --out",
                path.display()
            )));
        }
        fs::create_dir_all(path.join(ELITES))?;
        if !budget.admit(dir_size(&path)?) {
            return Err(LabError::Output(format!(
                "{} already exceeds the byte cap",
                path.display()
            )));
        }
        let rows = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.join(ROWS))?;
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
        fs::write(self.path.join(ELITES).join(format!("{name}.json")), content)?;
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
        fs::write(self.path.join(SUMMARY), content)?;
        Ok(())
    }
}

/// Git state of the injected root, as resolved by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitProvenance {
    /// The CLI's case: the root is a checkout at `revision`.
    Checkout { revision: String, dirty: bool },
    /// A library caller injected a root that is not a checkout.
    Unavailable,
}

impl GitProvenance {
    /// The summary's `(git_revision, dirty, git)` provenance fields.
    #[must_use]
    pub fn fields(&self) -> (Option<String>, Option<bool>, Option<GitMarker>) {
        match self {
            Self::Checkout { revision, dirty } => (Some(revision.clone()), Some(*dirty), None),
            Self::Unavailable => (None, None, Some(GitMarker::Unavailable)),
        }
    }
}

/// The root a run writes under (`<root>/.bench-artifacts/`) and its git
/// state. The library never resolves either; the caller injects them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabRoot {
    pub path: PathBuf,
    pub git: GitProvenance,
}

impl LabRoot {
    /// A root that is not a git checkout (tests, a mutation-testing copy).
    #[must_use]
    pub fn without_git(path: PathBuf) -> Self {
        Self {
            path,
            git: GitProvenance::Unavailable,
        }
    }
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

    /// A per-test root under the system temp directory, not a checkout.
    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("petri-lab-unit-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn out_must_stay_inside_bench_artifacts() {
        let root = temp_root("resolve-out");
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
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_directory_holding_prior_run_outputs_is_refused_before_any_write() {
        let path = temp_root("reuse");
        let budget = || Budget::new(64 << 20, SUMMARY_RESERVE).unwrap();
        let mut first = RunDir::create(path.clone(), budget()).unwrap();
        assert!(first.write_row("{\"row\":1}").unwrap());
        first.write_summary(b"{}").unwrap();
        drop(first);
        let before = fs::read(path.join("rows.ndjson")).unwrap();
        assert!(RunDir::create(path.clone(), budget()).is_err());
        assert_eq!(fs::read(path.join("rows.ndjson")).unwrap(), before);
        assert_eq!(fs::read(path.join("summary.json")).unwrap(), b"{}");
        // An existing directory without run outputs is still accepted.
        let empty = path.join("empty");
        fs::create_dir_all(&empty).unwrap();
        assert!(RunDir::create(empty, budget()).is_ok());
        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn the_default_cap_leaves_room_beyond_the_summary_reserve() {
        let mut budget = Budget::new(DEFAULT_BYTE_CAP, SUMMARY_RESERVE).unwrap();
        assert!(budget.admit(SUMMARY_RESERVE));
    }

    #[test]
    fn existing_bytes_and_each_row_newline_count_against_the_budget() {
        let path = temp_root("existing-bytes");
        fs::create_dir_all(path.join("notes/deeper")).unwrap();
        fs::write(path.join("notes/a.txt"), [0u8; 30]).unwrap();
        fs::write(path.join("notes/deeper/b.txt"), [0u8; 20]).unwrap();
        // 100 bytes for rows and elites; 50 already used by the notes.
        let mut dir = RunDir::create(path.clone(), Budget::new(200, 100).unwrap()).unwrap();
        // A 49-byte line plus its newline fills the remaining 50 exactly.
        assert!(dir.write_row(&"x".repeat(49)).unwrap());
        assert!(
            !dir.write_row("").unwrap(),
            "an empty row still costs 1 byte"
        );
        assert_eq!(fs::read(path.join("rows.ndjson")).unwrap().len(), 50);
        fs::remove_dir_all(&path).unwrap();

        let full = temp_root("existing-bytes-full");
        fs::write(full.join("big.bin"), [0u8; 101]).unwrap();
        assert!(RunDir::create(full.clone(), Budget::new(200, 100).unwrap()).is_err());
        fs::remove_dir_all(&full).unwrap();
    }

    #[test]
    fn an_elite_over_the_budget_is_refused_and_not_written() {
        let path = temp_root("elite-cap");
        let mut dir = RunDir::create(path.clone(), Budget::new(200, 100).unwrap()).unwrap();
        assert!(!dir.write_elite("big", &[b'x'; 101]).unwrap());
        assert!(!path.join("elites/big.json").exists());
        assert!(dir.write_elite("fits", &[b'x'; 100]).unwrap());
        assert!(path.join("elites/fits.json").exists());
        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn a_summary_may_fill_the_reserve_exactly() {
        let path = temp_root("summary-reserve");
        let mut dir = RunDir::create(path.clone(), Budget::new(200, 100).unwrap()).unwrap();
        assert!(dir.write_summary(&[b' '; 101]).is_err());
        assert!(!path.join("summary.json").exists());
        dir.write_summary(&[b' '; 100]).unwrap();
        assert_eq!(fs::read(path.join("summary.json")).unwrap().len(), 100);
        fs::remove_dir_all(&path).unwrap();
    }

    /// Independent civil calendar: whole days walked forward from 1970.
    fn reference_stamp(secs: u64) -> String {
        let leap =
            |y: u64| (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400);
        let mut days = secs / 86_400;
        let rem = secs % 86_400;
        let mut year = 1970;
        while days >= if leap(year) { 366 } else { 365 } {
            days -= if leap(year) { 366 } else { 365 };
            year += 1;
        }
        let lengths = [
            31,
            if leap(year) { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        let mut month = 1;
        for length in lengths {
            if days < length {
                break;
            }
            days -= length;
            month += 1;
        }
        format!(
            "{year:04}{month:02}{:02}T{:02}{:02}{:02}Z",
            days + 1,
            rem / 3_600,
            rem % 3_600 / 60,
            rem % 60
        )
    }

    #[test]
    fn the_reference_calendar_matches_known_dates() {
        assert_eq!(reference_stamp(0), "19700101T000000Z");
        assert_eq!(reference_stamp(951_782_400), "20000229T000000Z");
        assert_eq!(reference_stamp(1_709_251_199), "20240229T235959Z");
    }

    #[test]
    fn utc_stamp_is_the_current_utc_second() {
        let now = || {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        };
        let before = now();
        let stamp = utc_stamp();
        let after = now();
        assert!(
            (before..=after).any(|secs| reference_stamp(secs) == stamp),
            "{stamp} not within [{}, {}]",
            reference_stamp(before),
            reference_stamp(after)
        );
    }

    #[test]
    fn git_provenance_maps_to_the_summary_fields() {
        let checkout = GitProvenance::Checkout {
            revision: "abc".into(),
            dirty: true,
        };
        assert_eq!(checkout.fields(), (Some("abc".into()), Some(true), None));
        assert_eq!(
            GitProvenance::Unavailable.fields(),
            (None, None, Some(GitMarker::Unavailable))
        );
    }

    #[test]
    fn utc_stamp_has_the_directory_shape() {
        let stamp = utc_stamp();
        assert_eq!(stamp.len(), 16);
        assert!(stamp.starts_with("20") && stamp.ends_with('Z'));
    }
}
