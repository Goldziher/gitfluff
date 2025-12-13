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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BodyPolicy {
    #[default]
    Any,
    SingleLine,
    RequireBody,
}

#[derive(Debug, Default)]
pub struct LintOptions {
    pub message_pattern: Option<MessagePattern>,
    pub exclude_rules: Vec<ExcludeRule>,
    pub cleanup_rules: Vec<CleanupRule>,
    pub body_policy: BodyPolicy,
    pub enforce_conventional_spec: bool,
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

    let header_line = message.lines().next().unwrap_or("");
    if header_line.trim().is_empty() {
        violations.push("Commit message header must not be empty".to_string());
        return violations;
    }

    if let Some(pattern) = &options.message_pattern
        && !pattern.regex.is_match(header_line.trim())
    {
        let desc = pattern
            .description
            .as_deref()
            .unwrap_or("Commit message does not match required pattern");
        violations.push(desc.to_string());
    }

    if options.enforce_conventional_spec {
        violations.extend(validate_conventional_spec(message, options.body_policy));
    } else {
        violations.extend(validate_body_policy(message, options.body_policy));
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

#[derive(Debug)]
struct FooterEntry {
    token: String,
    value: String,
}

fn validate_body_policy(message: &str, policy: BodyPolicy) -> Vec<String> {
    match policy {
        BodyPolicy::Any => Vec::new(),
        BodyPolicy::SingleLine => {
            if message.lines().skip(1).any(|line| !line.trim().is_empty()) {
                vec!["Commit message must be a single line".to_string()]
            } else {
                Vec::new()
            }
        }
        BodyPolicy::RequireBody => {
            let mut lines = message.lines();
            lines.next(); // header

            let mut saw_blank = false;
            let mut body_has_content = false;

            for line in lines {
                if line.trim().is_empty() {
                    saw_blank = true;
                    continue;
                }
                if !saw_blank {
                    return vec![
                        "Body must begin with a blank line after the description".to_string(),
                    ];
                }
                body_has_content = true;
                break;
            }

            if !body_has_content {
                vec!["Commit message must include a body after a blank line".to_string()]
            } else {
                Vec::new()
            }
        }
    }
}

fn validate_conventional_spec(message: &str, policy: BodyPolicy) -> Vec<String> {
    let mut violations = Vec::new();
    let mut lines = message.lines();
    let header_line = lines.next().unwrap_or("");

    if let Err(err) = parse_header(header_line) {
        violations.push(err);
        return violations;
    }

    let remaining: Vec<&str> = lines.collect();
    let sections = parse_body_and_footers(&remaining, &mut violations);

    if policy == BodyPolicy::RequireBody && !sections.body_present {
        violations.push("Commit message must include a body after a blank line".to_string());
    }

    violations.extend(analyze_footers(&sections.footers));

    for footer in &sections.footers {
        let normalized = footer.token.replace('-', " ");
        if normalized.eq_ignore_ascii_case("BREAKING CHANGE") {
            if footer.token != "BREAKING CHANGE" && footer.token != "BREAKING-CHANGE" {
                violations.push(
                    "BREAKING CHANGE footer token must be uppercase (BREAKING CHANGE or BREAKING-CHANGE)"
                        .to_string(),
                );
            }
            if footer.value.trim().is_empty() {
                violations.push("BREAKING CHANGE footer must include a description".to_string());
            }
        }
    }

    violations
}

fn parse_header(header: &str) -> Result<(), String> {
    let colon_index = header.find(':').ok_or_else(|| {
        "Commit message header must look like `type: description` (optional `(scope)` and/or `!`)".to_string()
    })?;

    if !header[colon_index..].starts_with(": ") {
        return Err(
            "Commit message header must use `type: description` with a colon and space".to_string(),
        );
    }

    if colon_index == 0 {
        return Err("Commit message type must not be empty".to_string());
    }

    let description = &header[(colon_index + 2)..];
    if description.trim().is_empty() {
        return Err("Commit message description must not be empty".to_string());
    }

    let mut prefix = &header[..colon_index];
    let breaking_by_bang = if prefix.ends_with('!') {
        prefix = &prefix[..prefix.len() - 1];
        true
    } else {
        false
    };

    let mut scope = None;
    if let Some(scope_start) = prefix.find('(') {
        if !prefix.ends_with(')') {
            return Err("Commit scope, when present, must be wrapped in parentheses".to_string());
        }
        let raw_scope = &prefix[(scope_start + 1)..(prefix.len() - 1)];
        if raw_scope.trim().is_empty() {
            return Err("Commit scope must not be empty".to_string());
        }
        scope = Some(raw_scope.to_string());
        prefix = &prefix[..scope_start];
    }

    if prefix.trim().is_empty() {
        return Err("Commit message type must not be empty".to_string());
    }

    if !prefix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(
            "Commit message type must be a single word (letters, numbers, or underscore)"
                .to_string(),
        );
    }

    let _commit_type = prefix.to_lowercase();
    let _scope = scope;
    let _breaking = breaking_by_bang;
    let _description = description;

    Ok(())
}

struct Sections {
    body_present: bool,
    footers: Vec<FooterEntry>,
}

fn parse_body_and_footers(rest_lines: &[&str], violations: &mut Vec<String>) -> Sections {
    let mut body_present = false;
    let mut prev_blank = false;
    let mut current_footer: Option<FooterEntry> = None;
    let mut footers = Vec::new();

    for raw_line in rest_lines {
        let line = raw_line.trim_end_matches('\r');
        let is_blank = line.trim().is_empty();

        if let Some(footer) = current_footer.as_mut() {
            if is_blank {
                if !footer.value.is_empty() {
                    footer.value.push('\n');
                }
                prev_blank = true;
                continue;
            }
            if let Some(new_footer) = parse_footer_line(line) {
                footers.push(current_footer.take().unwrap());
                current_footer = Some(new_footer);
                prev_blank = false;
                continue;
            }

            if !footer.value.is_empty() {
                footer.value.push('\n');
            }
            footer.value.push_str(line);
            prev_blank = false;
            continue;
        }

        if is_blank {
            prev_blank = true;
            continue;
        }

        if let Some(new_footer) = parse_footer_line(line) {
            if !prev_blank {
                violations.push(
                    "Footers must be separated from the summary/body by a blank line".to_string(),
                );
            }
            current_footer = Some(new_footer);
            prev_blank = false;
            continue;
        }

        if !body_present && !prev_blank {
            violations.push("Body must begin with a blank line after the description".to_string());
        }

        body_present = true;
        prev_blank = false;
    }

    if let Some(footer) = current_footer.take() {
        footers.push(footer);
    }

    Sections {
        body_present,
        footers,
    }
}

fn parse_footer_line(line: &str) -> Option<FooterEntry> {
    if let Some(idx) = line.find(": ") {
        let token = line[..idx].to_string();
        let value = line[(idx + 2)..].to_string();
        Some(FooterEntry { token, value })
    } else if let Some(idx) = line.find(" #") {
        let token = line[..idx].to_string();
        let value = line[(idx + 2)..].to_string();
        Some(FooterEntry { token, value })
    } else {
        None
    }
}

fn analyze_footers(footers: &[FooterEntry]) -> Vec<String> {
    let mut violations = Vec::new();

    for footer in footers {
        let token_trimmed = footer.token.trim();
        if token_trimmed.is_empty() {
            violations.push("Footer token must not be empty".to_string());
            continue;
        }

        let normalized = token_trimmed.replace('-', " ");

        if normalized.eq_ignore_ascii_case("BREAKING CHANGE") {
            // handled separately in caller
            continue;
        }

        if token_trimmed.chars().any(|c| c.is_whitespace()) {
            violations.push(format!(
                "Footer token `{}` must use hyphen in place of whitespace",
                token_trimmed
            ));
        }

        if !token_trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            violations.push(format!(
                "Footer token `{}` must use alphanumeric characters or hyphen",
                token_trimmed
            ));
        }
    }

    violations
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

    #[test]
    fn conventional_commit_with_body_and_footer_is_valid() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>[A-Za-z]+)(\\((?P<scope>[^)]+)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat(parser): support pipes\n\nAdd parsing for foo | bar\n\nRefs: 123";
        let outcome = lint_message(message, &options);
        assert!(
            outcome.violations_before.is_empty(),
            "expected no violations, got {:?}",
            outcome.violations_before
        );
    }

    #[test]
    fn conventional_commit_requires_blank_line_before_body() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>[A-Za-z]+)(\\((?P<scope>[^)]+)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat: add api\nbody without separator";
        let outcome = lint_message(message, &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("Body must begin with a blank line")),
            "expected blank-line violation"
        );
    }

    #[test]
    fn footers_require_blank_line() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>[A-Za-z]+)(\\((?P<scope>[^)]+)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat: adjust login\nBREAKING CHANGE: password flow updated";
        let outcome = lint_message(message, &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("Footers must be separated")),
            "expected footer separation violation"
        );
    }

    #[test]
    fn breaking_change_footer_requires_description() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>[A-Za-z]+)(\\((?P<scope>[^)]+)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat!: add api\n\nBREAKING CHANGE: ";
        let outcome = lint_message(message, &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("BREAKING CHANGE footer must include a description")),
            "expected breaking change description violation"
        );
    }

    #[test]
    fn breaking_change_token_must_be_uppercase() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>[A-Za-z]+)(\\((?P<scope>[^)]+)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat: add option\n\nbreaking change: not uppercase";
        let outcome = lint_message(message, &options);
        assert!(
            outcome
                .violations_before
                .iter()
                .any(|msg| msg.contains("BREAKING CHANGE footer token must be uppercase")),
            "expected uppercase violation"
        );
    }
}
