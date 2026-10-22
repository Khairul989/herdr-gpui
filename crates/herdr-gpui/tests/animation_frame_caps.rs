#![allow(clippy::unwrap_used, clippy::expect_used)]
//! A looping GPUI animation runs its own timer and redraws its view on it,
//! out of step with every other loop, so a few spinners redraw the window
//! many times a tick. Every loop in the app runs on the shared clock in
//! `animation_clock.rs` instead, which redraws all of them once a tick.

use std::{
    fs,
    path::{Path, PathBuf},
};

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if name != "tests" {
                sources(&path, found);
            }
        } else if name.ends_with(".rs") && name != "tests.rs" {
            found.push(path);
        }
    }
}

/// `text` without its `//` comments, so prose about animations is not read
/// as code. Line breaks are kept, so line numbers still match the file.
fn code(text: &str) -> String {
    text.lines()
        .map(|line| line.find("//").map_or(line, |comment| &line[..comment]))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_looping_animation_runs_on_the_shared_clock() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let (mut shared, mut own) = (0, Vec::new());
    for file in files {
        let text = code(&fs::read_to_string(&file).unwrap());
        shared += text.matches("animation_clock::looping(").count();
        // GPUI's loop builders take no argument, unlike `str::repeat`.
        for pattern in [".repeat()", ".repeat_synced()"] {
            for (start, _) in text.match_indices(pattern) {
                let line = text[..start].lines().count();
                own.push(format!("{}:{line}", file.display()));
            }
        }
    }
    // A scanner that finds nothing would pass a tree full of GPUI loops.
    assert!(shared >= 10, "found only {shared} shared-clock loops");
    assert!(
        own.is_empty(),
        "looping animations with timers of their own: {own:#?}"
    );
}
