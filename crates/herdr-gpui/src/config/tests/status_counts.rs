use super::*;

/// An unset key draws the rows' own dots; every style reads back from its
/// own spelling, and nothing else is a style.
#[test]
fn status_counts_default_to_dots_and_read_every_style() -> anyhow::Result<()> {
    assert_eq!(Config::default().status_counts, StatusCounts::Dots);
    assert_eq!(Config::parse("")?.status_counts, StatusCounts::Dots);
    assert_eq!(
        Config::parse(DEFAULT_CONFIG)?.status_counts,
        StatusCounts::Dots
    );
    for &style in StatusCounts::ALL {
        let text = format!("status_counts = '{}'", style.as_str());
        assert_eq!(Config::parse(&text)?.status_counts, style);
    }
    for invalid in ["Solid", "", "bad"] {
        assert!(Config::parse(&format!("status_counts = '{invalid}'")).is_err());
    }
    Ok(())
}

#[test]
fn status_counts_preferences_write_their_key_in_place() -> anyhow::Result<()> {
    use crate::config::preferences::Preference;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    fs::write(
        &path,
        "# keep\nfuture = 1\nstatus_counts = 'split' # mine\n",
    )?;
    Config::save_preference_path(Preference::StatusCounts(StatusCounts::Chips), &path)?;
    let text = fs::read_to_string(&path)?;
    assert!(text.contains("# keep\nfuture = 1"), "{text}");
    assert!(text.contains("status_counts = \"chips\" # mine"), "{text}");
    assert_eq!(Config::parse(&text)?.status_counts, StatusCounts::Chips);
    Ok(())
}
