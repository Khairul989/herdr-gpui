//! Opt-in redraw counters for power benchmarks. With `HERDR_FRAME_STATS` set
//! to a file path, a background thread appends one line a second: Unix
//! milliseconds, then how often each [`Counter`] fired since the previous
//! line. Unset, counting costs one relaxed load.
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Write,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

macro_rules! counters {
    ($($variant:ident => $name:literal,)*) => {
        /// Something that costs a wakeup or a redraw.
        #[derive(Clone, Copy, Debug)]
        pub(crate) enum Counter {
            $($variant,)*
        }

        const NAMES: &[&str] = &[$($name,)*];

        #[cfg(test)]
        const VARIANTS: &[&str] = &[$(stringify!($variant),)*];
    };
}

counters! {
    LayerRender => "layer_renders",
    Paint => "terminal_paints",
    RootRender => "root_renders",
    SidebarRender => "sidebar_renders",
    Tick => "ticks",
}

const INTERVAL: Duration = Duration::from_secs(1);
static STATS: Stats = Stats::new();

struct Stats {
    enabled: AtomicBool,
    counts: [AtomicU64; NAMES.len()],
}

impl Stats {
    const fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            counts: [const { AtomicU64::new(0) }; NAMES.len()],
        }
    }

    fn count(&self, counter: Counter) {
        if self.enabled.load(Ordering::Relaxed) {
            self.counts[counter as usize].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Every count since the last call, in [`NAMES`] order, reset to zero.
    fn drain(&self) -> Vec<u64> {
        self.counts
            .iter()
            .map(|count| count.swap(0, Ordering::Relaxed))
            .collect()
    }
}

pub(crate) fn count(counter: Counter) {
    STATS.count(counter);
}

/// Starts reporting when `HERDR_FRAME_STATS` names a writable file.
pub(crate) fn start() {
    let Some(path) = std::env::var_os("HERDR_FRAME_STATS").filter(|path| !path.is_empty()) else {
        return;
    };
    if let Err(error) = report_to(&path) {
        tracing::warn!(?path, %error, "Frame stats unavailable");
    }
}

fn report_to(path: &OsString) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "unix_ms {}", NAMES.join(" "))?;
    thread::Builder::new()
        .name("herdr-frame-stats".into())
        .spawn(move || report(file))?;
    // Only once a reader exists, so a failed spawn leaves counting off.
    STATS.enabled.store(true, Ordering::Relaxed);
    Ok(())
}

fn report(mut file: File) {
    loop {
        thread::sleep(INTERVAL);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        if let Err(error) = writeln!(file, "{}", line(now, &STATS.drain())) {
            STATS.enabled.store(false, Ordering::Relaxed);
            tracing::warn!(%error, "Frame stats stopped");
            return;
        }
    }
}

fn line(unix_ms: u128, counts: &[u64]) -> String {
    let mut line = unix_ms.to_string();
    for count in counts {
        line.push(' ');
        line.push_str(&count.to_string());
    }
    line
}

#[cfg(test)]
mod tests;
