use super::*;

/// Herdr's `agent_brand_color` (`src/ui/sidebar/agent_logo.rs`): the only
/// agents with a verified brand colour. Every other mark keeps the theme tint.
const HERDR_BRANDS: &[(&str, u32)] = &[
    ("agy", 0x4285F4),
    ("amp", 0xFF5543),
    ("claude", 0xD97757),
    ("codex", 0x10A37F),
    ("copilot", 0x8957E5),
    ("devin", 0x2E6CF6),
    ("gemini", 0x4285F4),
    ("kiro", 0x790EF7),
];

#[test]
fn branded_marks_use_herdrs_colours_and_the_rest_keep_the_theme_tint() {
    const THEME: u32 = 0x123456;
    for &icon in AgentIcon::ALL {
        let label = icon
            .path()
            .trim_start_matches("icons/agent-")
            .trim_end_matches(".svg");
        let brand = HERDR_BRANDS
            .iter()
            .find(|(name, _)| *name == label)
            .map(|&(_, colour)| colour);
        assert_eq!(icon.brand_color(), brand, "{label}");
        assert_eq!(icon.tint(THEME), brand.unwrap_or(THEME), "{label}");
    }
    // Each listed brand names a real mark, so a typo cannot pass silently.
    for (name, _) in HERDR_BRANDS {
        assert_ne!(
            AgentIcon::from_identity(Some(name)),
            AgentIcon::Generic,
            "{name}"
        );
    }
    assert_eq!(AgentIcon::Generic.tint(THEME), THEME);
}
