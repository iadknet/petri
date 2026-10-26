//! The frontend's generated wire types (T30.F01 invariant 10).
//!
//! One test renders every root of [`render_roots`] and its dependencies with
//! `ts-rs`, one file per type, and compares the declarations with the
//! committed files under `frontend/src/types/generated/`, whitespace, member
//! separators, leading union bars and import lines normalized (Biome formats
//! and sorts the committed files). It fails on any added, removed, renamed or
//! retyped type, field or variant and lists the differing files. With
//! `PETRI_WRITE_TS_TYPES=1` it rewrites them instead; `npm run lint:fix` then
//! formats them. `cargo test` without the flag never writes.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ts_rs::{Config, TypeVisitor, TS};

/// Every rendered file, by output path: the type that rendered it and its
/// text.
struct Render<'a> {
    cfg: &'a Config,
    files: BTreeMap<PathBuf, (TypeId, &'static str, String)>,
    collisions: Vec<String>,
}

impl TypeVisitor for Render<'_> {
    fn visit<T: TS + 'static + ?Sized>(&mut self) {
        let Some(path) = T::output_path() else {
            return;
        };
        let id = TypeId::of::<T>();
        if let Some((seen, name, _)) = self.files.get(&path) {
            if *seen != id {
                self.collisions.push(format!(
                    "{} is rendered by both {name} and {}",
                    path.display(),
                    std::any::type_name::<T>()
                ));
            }
            return;
        }
        let text = T::export_to_string(self.cfg)
            .unwrap_or_else(|error| panic!("{}: {error}", std::any::type_name::<T>()));
        self.files
            .insert(path, (id, std::any::type_name::<T>(), text));
        T::visit_dependencies(self);
        T::visit_generics(self);
    }
}

/// The root set: every request, response and WebSocket message type.
fn render_roots(render: &mut Render<'_>) {
    render.visit::<crate::transport::protocol::ServerMessage>();
    render.visit::<crate::transport::protocol::ClientMessage>();
}

/// A declaration with formatting removed: import lines dropped, whitespace
/// removed, `;` member separators read as `,`, trailing separators and
/// leading union bars dropped.
fn normalize(text: &str) -> String {
    let body: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("import "))
        .flat_map(str::chars)
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == ';' { ',' } else { c })
        .collect();
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            (',', Some('}' | ')' | ']' | '>')) => {}
            ('|', _) if matches!(out.chars().last(), Some('=' | ':' | '(' | '<' | ',' | '[')) => {}
            ('"', _) if matches!(out.chars().last(), Some('{' | ',')) => {
                // A quoted property key, which Biome unquotes when it is an
                // identifier.
                let key: String = chars.by_ref().take_while(|&c| c != '"').collect();
                let identifier = key.chars().all(|c| c.is_alphanumeric() || c == '_');
                if identifier {
                    out.push_str(&key);
                } else {
                    out.push('"');
                    out.push_str(&key);
                    out.push('"');
                }
            }
            _ => out.push(c),
        }
    }
    out.trim_end_matches(',').to_owned()
}

fn generated_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend/src/types/generated")
}

#[test]
fn generated_types_match_the_wire_types() {
    let cfg = Config::from_env();
    assert_eq!(
        cfg.large_int(),
        "number",
        "TS_RS_LARGE_INT comes from .cargo/config.toml"
    );
    let mut render = Render {
        cfg: &cfg,
        files: BTreeMap::new(),
        collisions: Vec::new(),
    };
    render_roots(&mut render);
    assert!(render.collisions.is_empty(), "{:#?}", render.collisions);

    let dir = generated_dir();
    let write = std::env::var("PETRI_WRITE_TS_TYPES").as_deref() == Ok("1");
    let committed: BTreeMap<PathBuf, String> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .map(|entry| entry.expect("a directory entry").path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
                .map(|path| {
                    let text = std::fs::read_to_string(&path).expect("a readable file");
                    (PathBuf::from(path.file_name().expect("a file name")), text)
                })
                .collect()
        })
        .unwrap_or_default();

    let mut differing = Vec::new();
    for (path, (_, _, text)) in &render.files {
        if committed
            .get(path)
            .is_none_or(|old| normalize(old) != normalize(text))
        {
            differing.push(path.display().to_string());
            if write {
                std::fs::create_dir_all(&dir).expect("the generated directory");
                std::fs::write(dir.join(path), text).expect("a written file");
            }
        }
    }
    for path in committed
        .keys()
        .filter(|path| !render.files.contains_key(*path))
    {
        differing.push(format!("{} (no longer rendered)", path.display()));
        if write {
            std::fs::remove_file(dir.join(path)).expect("a removed file");
        }
    }
    assert!(
        write || differing.is_empty(),
        "frontend/src/types/generated differs from the wire types in {differing:#?}; \
         run `PETRI_WRITE_TS_TYPES=1 cargo test -p v3-server generated_types` then \
         `npm run lint:fix` in frontend/"
    );
}

#[test]
fn normalize_ignores_biome_formatting_but_not_types() {
    let raw = "import type { A } from \"./A.ts\";\n\nexport type T = { \"type\": \"t\", b: A, };\n\
               export type U = \"X\" | \"Y\";\n";
    let biome =
        "import type { A } from \"./A.ts\";\n\nexport type T = {\n\ttype: \"t\";\n\tb: A;\n};\n\
                 export type U =\n\t| \"X\"\n\t| \"Y\";\n";
    assert_eq!(normalize(raw), normalize(biome));
    assert_ne!(normalize(raw), normalize(&raw.replace("\"t\"", "\"u\"")));
    assert_ne!(normalize(raw), normalize(&raw.replace("b: A", "c: A")));
}
