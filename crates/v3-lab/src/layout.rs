//! Arena layout files (`--layout file.json`): `{"arena_format": 1, "rows":
//! [...]}`, square, 16–128 rows of `.` empty, `#` barrier, `F` food (at
//! least one) and `S` start (exactly one). The row count is the arena size.

use std::path::Path;

use v3_core::contracts::Position;

use crate::LabError;

/// The only layout format.
pub const ARENA_FORMAT: u32 = 1;
/// Row-count bounds (the arena size).
pub const MIN_ROWS: usize = 16;
pub const MAX_ROWS: usize = 128;

/// The file as written.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutFile {
    pub arena_format: u32,
    pub rows: Vec<String>,
}

/// A parsed layout. Food and barrier cells are in row-major world order
/// (y, then x).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The path as given, recorded in the arena descriptor.
    pub path: String,
    pub rows: Vec<String>,
    pub start: Position,
    pub food: Vec<Position>,
    pub barriers: Vec<Position>,
}

impl Layout {
    /// Load and parse a layout file.
    ///
    /// # Errors
    ///
    /// I/O failures, malformed JSON, unknown keys or an invalid layout.
    pub fn load(path: &Path) -> Result<Self, LabError> {
        let file: LayoutFile = crate::read_json(path)?;
        Self::parse(path.display().to_string(), file)
            .map_err(|error| LabError::Config(format!("{}: {error}", path.display())))
    }

    /// Validate `file`.
    ///
    /// # Errors
    ///
    /// A message naming the first violated rule.
    pub fn parse(path: String, file: LayoutFile) -> Result<Self, String> {
        if file.arena_format != ARENA_FORMAT {
            return Err(format!(
                "arena_format {} is not {ARENA_FORMAT}",
                file.arena_format
            ));
        }
        let size = file.rows.len();
        if !(MIN_ROWS..=MAX_ROWS).contains(&size) {
            return Err(format!("{size} rows; a layout has {MIN_ROWS}..={MAX_ROWS}"));
        }
        let (mut start, mut food, mut barriers) = (Vec::new(), Vec::new(), Vec::new());
        for (y, row) in file.rows.iter().enumerate() {
            if row.chars().count() != size {
                return Err(format!(
                    "row {y} has {} cells; a layout is square ({size})",
                    row.chars().count()
                ));
            }
            for (x, cell) in row.chars().enumerate() {
                let position = Position::new(
                    u16::try_from(x).expect("x < 128"),
                    u16::try_from(y).expect("y < 128"),
                );
                match cell {
                    '.' => {}
                    '#' => barriers.push(position),
                    'F' => food.push(position),
                    'S' => start.push(position),
                    other => return Err(format!("row {y} holds `{other}`; use . # F S")),
                }
            }
        }
        let [start] = start[..] else {
            return Err(format!("{} start cells; a layout has one", start.len()));
        };
        if food.is_empty() {
            return Err("no food cell".into());
        }
        Ok(Self {
            path,
            rows: file.rows,
            start,
            food,
            barriers,
        })
    }

    /// The arena size: the row count.
    #[must_use]
    pub fn size(&self) -> u16 {
        u16::try_from(self.rows.len()).expect("at most 128 rows")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A `size`-row layout of `.` with `cells` (x, y, char) set.
    pub(crate) fn rows(size: usize, cells: &[(usize, usize, char)]) -> Vec<String> {
        let mut grid = vec![vec!['.'; size]; size];
        for &(x, y, c) in cells {
            grid[y][x] = c;
        }
        grid.into_iter()
            .map(|row| row.into_iter().collect())
            .collect()
    }

    fn parse(rows: Vec<String>) -> Result<Layout, String> {
        Layout::parse(
            "t.json".into(),
            LayoutFile {
                arena_format: 1,
                rows,
            },
        )
    }

    #[test]
    fn cells_parse_in_row_major_order() {
        let layout = parse(rows(
            16,
            &[
                (3, 2, 'F'),
                (1, 2, 'F'),
                (5, 1, 'F'),
                (8, 8, 'S'),
                (0, 0, '#'),
            ],
        ))
        .unwrap();
        assert_eq!(layout.size(), 16);
        assert_eq!(layout.start, Position::new(8, 8));
        assert_eq!(
            layout.food,
            [
                Position::new(5, 1),
                Position::new(1, 2),
                Position::new(3, 2)
            ]
        );
        assert_eq!(layout.barriers, [Position::new(0, 0)]);
    }

    #[test]
    fn invalid_layouts_are_refused() {
        let ok = |cells: &[(usize, usize, char)]| rows(16, cells);
        let base = [(1, 1, 'S'), (5, 5, 'F')];
        assert!(parse(ok(&base)).is_ok());
        assert!(parse(ok(&[(1, 1, 'S')])).is_err(), "no food");
        assert!(parse(ok(&[(5, 5, 'F')])).is_err(), "no start");
        assert!(parse(ok(&[(1, 1, 'S'), (2, 2, 'S'), (5, 5, 'F')])).is_err());
        assert!(parse(ok(&[(1, 1, 'S'), (5, 5, 'F'), (6, 6, 'x')])).is_err());
        assert!(parse(rows(15, &base)).is_err(), "below 16 rows");
        assert!(parse(rows(129, &base)).is_err(), "above 128 rows");
        let mut ragged = ok(&base);
        ragged[3].push('.');
        assert!(parse(ragged).is_err(), "not square");
        let format = Layout::parse(
            "t.json".into(),
            LayoutFile {
                arena_format: 2,
                rows: ok(&base),
            },
        );
        assert!(format.is_err());
        let unknown: Result<LayoutFile, _> =
            serde_json::from_str(r#"{"arena_format": 1, "rows": [], "terrain": []}"#);
        assert!(unknown.is_err(), "unknown keys are refused");
    }
}
