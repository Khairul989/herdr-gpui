#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn counts_only_while_enabled_and_drain_resets() {
    let stats = Stats::new();
    stats.count(Counter::Tick);
    assert!(stats.drain().iter().all(|count| *count == 0));

    stats.enabled.store(true, Ordering::Relaxed);
    stats.count(Counter::Tick);
    stats.count(Counter::Tick);
    stats.count(Counter::RootRender);
    let counts = stats.drain();
    let at = |name| counts[NAMES.iter().position(|n| *n == name).unwrap()];
    assert_eq!(at("ticks"), 2);
    assert_eq!(at("root_renders"), 1);
    assert_eq!(counts.iter().sum::<u64>(), 3);
    assert!(stats.drain().iter().all(|count| *count == 0));
}

#[test]
fn a_line_is_the_time_then_one_count_per_name() {
    let counts: Vec<u64> = (1..=NAMES.len() as u64).collect();
    let line = line(1_700_000_000_123, &counts);
    let fields: Vec<&str> = line.split(' ').collect();
    assert_eq!(fields[0], "1700000000123");
    assert_eq!(fields.len(), NAMES.len() + 1);
    assert_eq!(fields[NAMES.len()], NAMES.len().to_string());
}

/// A deleted `count(..)` call would read as a column of zeros, which a
/// before-and-after comparison takes for an improvement.
#[test]
fn every_counter_is_counted_somewhere() {
    fn sources(dir: &std::path::Path, text: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if !path.ends_with("frame_stats") {
                    sources(&path, text);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                text.push_str(&std::fs::read_to_string(&path).unwrap());
            }
        }
    }
    let mut text = String::new();
    sources(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut text,
    );
    assert_eq!(VARIANTS.len(), NAMES.len());
    for variant in VARIANTS {
        assert!(
            text.contains(&format!("frame_stats::Counter::{variant})")),
            "Counter::{variant} is never counted"
        );
    }
}
