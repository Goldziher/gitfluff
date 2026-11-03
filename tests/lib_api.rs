use std::path::Path;

use gitfluff::config::{ExcludeRuleConfig, FileConfig};
use gitfluff::{LintSettings, lint_with_config, resolve_lint_options};

fn config_from(toml_source: &str) -> FileConfig {
    toml::from_str(toml_source).expect("valid config")
}

#[test]
fn accepts_conventional_message_without_config() {
    let outcome = lint_with_config("feat(api): add login\n", None, &LintSettings::default()).unwrap();
    assert!(outcome.violations_before.is_empty());
}

#[test]
fn rejects_non_conventional_message() {
    let outcome = lint_with_config("add login\n", None, &LintSettings::default()).unwrap();
    assert!(!outcome.violations_before.is_empty());
}

#[test]
fn config_require_body_is_enforced() {
    let config = config_from("[rules]\nrequire_body = true\n");
    let path = Path::new(".gitfluff.toml");
    let outcome = lint_with_config("feat: add login\n", Some((path, &config)), &LintSettings::default()).unwrap();
    assert!(!outcome.violations_before.is_empty());

    let outcome = lint_with_config(
        "feat: add login\n\nExplain why.\n",
        Some((path, &config)),
        &LintSettings::default(),
    )
    .unwrap();
    assert!(outcome.violations_before.is_empty());
}

#[test]
fn settings_preset_overrides_config_preset() {
    let config = config_from("preset = \"conventional\"\n");
    let settings = LintSettings {
        preset: Some("simple".to_string()),
        ..Default::default()
    };
    let outcome = lint_with_config("Add login\n", Some((Path::new(".gitfluff.toml"), &config)), &settings).unwrap();
    assert!(outcome.violations_before.is_empty());
}

#[test]
fn unknown_preset_is_an_error() {
    let settings = LintSettings {
        preset: Some("nope".to_string()),
        ..Default::default()
    };
    let error = resolve_lint_options(None, &settings).unwrap_err();
    assert!(error.to_string().contains("unknown preset `nope`"));
}

#[test]
fn single_line_and_require_body_in_config_conflict() {
    let config = config_from("[rules]\nsingle_line = true\nrequire_body = true\n");
    let error = resolve_lint_options(Some((Path::new("c.toml"), &config)), &LintSettings::default()).unwrap_err();
    assert!(error.to_string().contains("single_line"));
}

#[test]
fn builtin_ai_cleanup_strips_attribution_and_can_be_disabled() {
    let message = "feat: add login\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n";

    let resolved = resolve_lint_options(None, &LintSettings::default()).unwrap();
    let outcome = gitfluff::lint_message(message, &resolved.options);
    assert!(!outcome.cleaned_message.contains("Co-Authored-By"));
    assert!(!outcome.violations_before.is_empty());

    let settings = LintSettings {
        no_ai_cleanup: true,
        ..Default::default()
    };
    let resolved = resolve_lint_options(None, &settings).unwrap();
    let outcome = gitfluff::lint_message(message, &resolved.options);
    assert!(outcome.violations_before.is_empty());
}

#[test]
fn settings_excludes_are_applied() {
    let settings = LintSettings {
        excludes: vec![ExcludeRuleConfig {
            pattern: "WIP".to_string(),
            message: Some("no WIP".to_string()),
        }],
        ..Default::default()
    };
    let outcome = lint_with_config("feat: WIP login\n", None, &settings).unwrap();
    assert!(outcome.violations_before.iter().any(|v| v.contains("no WIP")));
}

#[test]
fn write_comes_from_config_or_settings() {
    let config = config_from("write = true\n");
    let resolved = resolve_lint_options(Some((Path::new("c.toml"), &config)), &LintSettings::default()).unwrap();
    assert!(resolved.write);
    assert!(resolved.options.autofix);

    let resolved = resolve_lint_options(None, &LintSettings::default()).unwrap();
    assert!(!resolved.write);
}

#[test]
fn generated_subjects_are_detected() {
    assert!(gitfluff::has_generated_subject("fixup! feat: x\n"));
    assert!(gitfluff::has_generated_subject("Revert \"feat: x\"\n"));
    assert!(!gitfluff::has_generated_subject("feat: x\n"));
}

#[test]
fn load_config_finds_nothing_in_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    assert!(gitfluff::load_config(None, dir.path()).unwrap().is_none());
}
