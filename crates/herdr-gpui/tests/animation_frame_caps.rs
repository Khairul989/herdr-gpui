#![allow(clippy::unwrap_used, clippy::expect_used)]
//! A looping GPUI animation without a frame-rate cap asks for a frame on
//! every display refresh, up to 120 a second, and re-renders the view
//! drawing it and every view above it. Each one in the app must cap its
//! rate with `with_max_fps`.

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
fn every_looping_animation_caps_its_frame_rate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    let (mut animations, mut uncapped) = (0, Vec::new());
    for file in files {
        let text = code(&fs::read_to_string(&file).unwrap());
        for (start, _) in text.match_indices("Animation::new(") {
            animations += 1;
            // The animation is configured before the animator closure that
            // follows it as the next argument.
            let rest = &text[start..];
            let chain = &rest[..rest.find('|').unwrap_or(rest.len())];
            if chain.contains(".repeat") && !chain.contains("with_max_fps") {
                let line = text[..start].lines().count();
                uncapped.push(format!("{}:{line}", file.display()));
            }
        }
    }
    // A scanner that finds nothing would pass a tree full of uncapped loops.
    assert!(animations >= 10, "found only {animations} animations");
    assert!(
        uncapped.is_empty(),
        "uncapped looping animations: {uncapped:#?}"
    );
}
