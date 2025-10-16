use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct MessagePattern {
    pub regex: Regex,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExcludeRule {
    pub regex: Regex,
    pub message: Option<String>,
    pub pattern_source: String,
}

#[derive(Debug, Clone)]
pub struct CleanupRule {
    pub regex: Regex,
    pub replace: String,
    pub description: Option<String>,
    pub pattern_source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyPolicy {
    Any,
    SingleLine,
    RequireBody,
}

impl Default for BodyPolicy {
    fn default() -> Self {
        BodyPolicy::Any
    }
}

#[derive(Debug, Default)]
pub struct LintOptions {
    pub message_pattern: Option<MessagePattern>,
    pub exclude_rules: Vec<ExcludeRule>,
    pub cleanup_rules: Vec<CleanupRule>,
    pub body_policy: BodyPolicy,
}

#[derive(Debug)]
pub struct LintOutcome {
    pub violations_before: Vec<String>,
    pub violations_after: Vec<String>,
    pub cleaned_message: String,
    pub cleanup_summaries: Vec<String>,
}

pub fn lint_message(message: &str, options: &LintOptions) -> LintOutcome {
    let violations_before = evaluate_message(message, options);
    let (cleaned_message, cleanup_summaries) = apply_cleanup(message, &options.cleanup_rules);
    let violations_after = evaluate_message(&cleaned_message, options);

    LintOutcome {
        violations_before,
        violations_after,
        cleaned_message,
        cleanup_summaries,
    }
}

fn evaluate_message(message: &str, options: &LintOptions) -> Vec<String> {
    let mut violations = Vec::new();

    for exclude in &options.exclude_rules {
        if exclude.regex.is_match(message) {
            let msg = exclude.message.clone().unwrap_or_else(|| {
                format!(
                    "Commit message matches excluded pattern `{}`",
                    exclude.pattern_source
                )
            });
            violations.push(msg);
        }
    }

    let header = message.lines().next().unwrap_or("").trim();
    if header.is_empty() {
        violations.push("Commit message header must not be empty".to_string());
    } else if let Some(pattern) = &options.message_pattern {
        if !pattern.regex.is_match(header) {
            let desc = pattern
                .description
                .as_deref()
                .unwrap_or("Commit message does not match required pattern");
            violations.push(desc.to_string());
        }
    }

    match options.body_policy {
        BodyPolicy::Any => {}
        BodyPolicy::SingleLine => {
            if message.lines().skip(1).any(|line| !line.trim().is_empty()) {
                violations.push("Commit message must be a single line".to_string());
            }
        }
        BodyPolicy::RequireBody => {
            let mut lines = message.lines();
            let _ = lines.next();
            let mut has_blank_separator = false;
            let mut body_has_content = false;
            for line in lines {
                if !has_blank_separator {
                    if line.trim().is_empty() {
                        has_blank_separator = true;
                    }
                } else if !line.trim().is_empty() {
                    body_has_content = true;
                    break;
                }
            }
            if !(has_blank_separator && body_has_content) {
                violations
                    .push("Commit message must include a body after a blank line".to_string());
            }
        }
    }

    violations
}

fn apply_cleanup(input: &str, rules: &[CleanupRule]) -> (String, Vec<String>) {
    let mut current = input.to_string();
    let mut summaries = Vec::new();

    for rule in rules {
        let replaced = rule
            .regex
            .replace_all(&current, rule.replace.as_str())
            .to_string();
        if replaced != current {
            let summary = rule
                .description
                .clone()
                .unwrap_or_else(|| format!("Applied cleanup `{}`", rule.pattern_source));
            summaries.push(summary);
            current = replaced;
        }
    }

    (current, summaries)
}

pub fn build_message_pattern(pattern: &str, description: Option<String>) -> Result<MessagePattern> {
    let regex = Regex::new(pattern)
        .with_context(|| format!("invalid message pattern regex `{pattern}`"))?;
    Ok(MessagePattern { regex, description })
}

pub fn build_exclude_rule(pattern: &str, message: Option<String>) -> Result<ExcludeRule> {
    let regex =
        Regex::new(pattern).with_context(|| format!("invalid exclude regex `{pattern}`"))?;
    Ok(ExcludeRule {
        regex,
        message,
        pattern_source: pattern.to_string(),
    })
}

pub fn build_cleanup_rule(
    find: &str,
    replace: &str,
    description: Option<String>,
) -> Result<CleanupRule> {
    let regex = Regex::new(find).with_context(|| format!("invalid cleanup regex `{find}`"))?;
    Ok(CleanupRule {
        regex,
        replace: replace.to_string(),
        description,
        pattern_source: find.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_header() {
        let options = LintOptions::default();
        let outcome = lint_message("", &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("header must not be empty")),
            "expected empty header violation"
        );
    }

    #[test]
    fn enforces_message_pattern() {
        let pattern = build_message_pattern("^feat: .+$", None).unwrap();
        let mut options = LintOptions::default();
        options.message_pattern = Some(pattern);
        let outcome = lint_message("fix: nope", &options);
        assert_eq!(outcome.violations_before.len(), 1);
    }

    #[test]
    fn applies_cleanup_rules() {
        let cleanup =
            build_cleanup_rule("\\s+$", "", Some("trim trailing whitespace".into())).unwrap();
        let mut options = LintOptions::default();
        options.cleanup_rules.push(cleanup);
        let outcome = lint_message("feat: demo   \n", &options);
        assert_eq!(outcome.cleaned_message, "feat: demo");
        assert_eq!(outcome.cleanup_summaries.len(), 1);
        assert!(outcome.violations_after.is_empty());
    }

    #[test]
    fn excludes_patterns() {
        let exclude = build_exclude_rule("(?i)wip", Some("WIP commits disallowed".into())).unwrap();
        let mut options = LintOptions::default();
        options.exclude_rules.push(exclude);
        let outcome = lint_message("wip: tmp", &options);
        assert_eq!(outcome.violations_before, vec!["WIP commits disallowed"]);
    }

    #[test]
    fn enforces_single_line_policy() {
        let mut options = LintOptions::default();
        options.body_policy = BodyPolicy::SingleLine;
        let outcome = lint_message("feat: header\n\nbody line", &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("single line"))
        );
    }

    #[test]
    fn enforces_require_body_policy() {
        let mut options = LintOptions::default();
        options.body_policy = BodyPolicy::RequireBody;
        let outcome = lint_message("feat: header\n", &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("must include a body"))
        );

        let ok = lint_message("feat: header\n\nbody", &options);
        assert!(
            ok.violations_before
                .iter()
                .all(|msg| !msg.contains("must include a body"))
        );
    }
}
