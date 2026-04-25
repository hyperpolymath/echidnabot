// SPDX-License-Identifier: PMPL-1.0-or-later
//! Bot-directive resolution + cascade.
//!
//! Per the bit-4(c) decision: target-repo `.machine_readable/bot_directives/`
//! takes priority over the per-repo DB column, which takes priority over the
//! daemon-wide config default. The cascade is:
//!
//!   1. `.machine_readable/bot_directives/echidnabot.a2ml` (if present)
//!   2. `.machine_readable/bot_directives/all.a2ml`        (if present)
//!   3. `repositories.mode` column                         (per-repo DB)
//!   4. `BotMode::default()` = Verifier
//!
//! The directive content can be either A2ML (TOML-flavoured, post-2026-04-12
//! migration) or Scheme (legacy `.scm`). Both formats look for `(mode "X")`
//! or `mode = "X"` and accept the same four values.
//!
//! NOTE: this module does NOT fetch the directive from the target repo's
//! filesystem — that requires either cloning the repo locally (handled by the
//! sandboxed executor, currently empty) or hitting the platform API. Callers
//! are responsible for providing the directive content; if they pass `None`,
//! the cascade falls through to DB → default.

use crate::modes::BotMode;
use crate::store::models::Repository;

/// Parse an A2ML/TOML directive looking for `[bot]` table with `mode = "X"`.
/// Returns `None` if no directive can be parsed.
pub fn parse_a2ml_directive(content: &str) -> Option<BotMode> {
    let parsed: toml::Value = toml::from_str(content).ok()?;
    let mode_str = parsed
        .get("bot")
        .and_then(|t| t.get("mode"))
        .and_then(|m| m.as_str())?;
    serde_json::from_value(serde_json::Value::String(mode_str.to_lowercase())).ok()
}

/// Resolve the bot mode for a repo + given directive content.
///
/// Cascade:
///   1. directive content (echidnabot.a2ml > all.a2ml > .scm fallback)
///   2. repo.mode (DB column)
///   3. `BotMode::default()`
///
/// `directive_content` is the result of fetching the most specific directive
/// the caller could find. Callers should try `echidnabot.a2ml` first, fall
/// back to `all.a2ml`, and pass `None` if neither exists. The format is
/// detected from content (A2ML if it parses as TOML, else legacy Scheme).
pub fn resolve_mode(repo: &Repository, directive_content: Option<&str>) -> BotMode {
    if let Some(content) = directive_content {
        // Try A2ML first (post-2026-04-12 canonical), then legacy Scheme.
        if let Some(mode) = parse_a2ml_directive(content) {
            tracing::debug!("Mode {} resolved from A2ML directive", mode);
            return mode;
        }
        let scheme_mode = super::parse_mode_from_directive(content);
        // parse_mode_from_directive returns Verifier on parse failure. To
        // detect "no directive matched", check whether the content contained
        // anything mode-shaped at all.
        if content.to_lowercase().contains("mode") {
            tracing::debug!("Mode {} resolved from Scheme directive", scheme_mode);
            return scheme_mode;
        }
    }
    // Fall back to per-repo DB setting.
    tracing::debug!("Mode {} resolved from repository.mode (DB)", repo.mode);
    repo.mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::Platform;

    fn fixture_repo(mode: BotMode) -> Repository {
        let mut repo = Repository::new(Platform::GitHub, "owner".into(), "name".into());
        repo.mode = mode;
        repo
    }

    #[test]
    fn parses_a2ml_directive() {
        let content = r#"
            [bot]
            mode = "advisor"
        "#;
        assert_eq!(parse_a2ml_directive(content), Some(BotMode::Advisor));
    }

    #[test]
    fn parses_a2ml_each_mode() {
        for (s, expected) in [
            ("verifier", BotMode::Verifier),
            ("advisor", BotMode::Advisor),
            ("consultant", BotMode::Consultant),
            ("regulator", BotMode::Regulator),
        ] {
            let content = format!("[bot]\nmode = \"{}\"", s);
            assert_eq!(parse_a2ml_directive(&content), Some(expected));
        }
    }

    #[test]
    fn parses_a2ml_case_insensitive() {
        let content = r#"
            [bot]
            mode = "ADVISOR"
        "#;
        assert_eq!(parse_a2ml_directive(content), Some(BotMode::Advisor));
    }

    #[test]
    fn returns_none_for_invalid_a2ml() {
        // Not TOML at all
        assert_eq!(parse_a2ml_directive("not even close"), None);
    }

    #[test]
    fn returns_none_for_a2ml_without_bot_mode() {
        let content = r#"
            [other]
            field = "value"
        "#;
        assert_eq!(parse_a2ml_directive(content), None);
    }

    #[test]
    fn cascade_directive_a2ml_wins_over_db() {
        let repo = fixture_repo(BotMode::Verifier);
        let directive = r#"
            [bot]
            mode = "regulator"
        "#;
        assert_eq!(resolve_mode(&repo, Some(directive)), BotMode::Regulator);
    }

    #[test]
    fn cascade_directive_scheme_wins_over_db() {
        let repo = fixture_repo(BotMode::Verifier);
        let directive = r#"(echidnabot (mode "advisor"))"#;
        assert_eq!(resolve_mode(&repo, Some(directive)), BotMode::Advisor);
    }

    #[test]
    fn cascade_falls_back_to_db_when_no_directive() {
        let repo = fixture_repo(BotMode::Consultant);
        assert_eq!(resolve_mode(&repo, None), BotMode::Consultant);
    }

    #[test]
    fn cascade_falls_back_to_db_when_directive_has_no_mode() {
        let repo = fixture_repo(BotMode::Advisor);
        let directive = "(echidnabot (provers \"lean\" \"coq\"))"; // no mode
        // Scheme parser returns Verifier on no-match, but our resolver's
        // "contains 'mode'" check makes us NOT trust that fallback. So we
        // fall through to DB.
        assert_eq!(resolve_mode(&repo, Some(directive)), BotMode::Advisor);
    }

    #[test]
    fn cascade_default_is_verifier_when_db_mode_default() {
        // Repository::new sets mode to BotMode::default() (Verifier).
        let repo = Repository::new(Platform::GitHub, "owner".into(), "name".into());
        assert_eq!(resolve_mode(&repo, None), BotMode::Verifier);
    }
}
