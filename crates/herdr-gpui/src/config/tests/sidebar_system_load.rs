use super::*;
use crate::config::{preferences::Preference, status_bar::Detail};

#[test]
fn sidebar_system_load_is_named_and_defaults_to_detailed() -> anyhow::Result<()> {
    for config in [Config::default(), Config::parse(DEFAULT_CONFIG)?] {
        assert_eq!(config.sidebar_style.system_load, Detail::Detailed);
    }
    for (name, detail) in [("detailed", Detail::Detailed), ("compact", Detail::Compact)] {
        assert_eq!(detail.name(), name);
        let config = Config::parse(&format!("[sidebar]\nsystem_load = \"{name}\""))?;
        assert_eq!(config.sidebar_style.system_load, detail);
        // The status bar keeps its own choice.
        assert_eq!(config.status_bar.system_load, Detail::Detailed);
    }
    for bad in ["\"graphs\"", "\"Compact\"", "true"] {
        assert!(Config::parse(&format!("[sidebar]\nsystem_load = {bad}")).is_err());
    }
    Ok(())
}

#[test]
fn sidebar_system_load_round_trips_through_the_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("local.toml");
    fs::write(&path, "[sidebar]\nsystem_load = \"detailed\" # mine\n")?;
    Config::save_preference_path(Preference::SidebarSystemLoad(Detail::Compact), &path)?;
    let text = fs::read_to_string(&path)?;
    assert!(text.contains("system_load = \"compact\" # mine"), "{text}");
    assert_eq!(
        Config::parse(&text)?.sidebar_style.system_load,
        Detail::Compact
    );
    Ok(())
}
