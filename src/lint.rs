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
    pub autofix: bool,
}

#[derive(Debug)]
pub struct LintOutcome {
    pub violations_before: Vec<String>,
    pub violations_after: Vec<String>,
    pub warnings_before: Vec<String>,
    pub warnings_after: Vec<String>,
    pub cleaned_message: String,
    pub cleanup_summaries: Vec<String>,
}

pub fn lint_message(message: &str, options: &LintOptions) -> LintOutcome {
    let (violations_before, warnings_before) = evaluate_message(message, options);
    let (mut cleaned_message, mut cleanup_summaries) =
        apply_cleanup(message, &options.cleanup_rules);
    if options.autofix {
        let (formatted, mut format_summaries) =
            apply_autofix(&cleaned_message, options.enforce_conventional_spec);
        if formatted != cleaned_message {
            cleaned_message = formatted;
        }
        cleanup_summaries.append(&mut format_summaries);
    }
    let (violations_after, warnings_after) = evaluate_message(&cleaned_message, options);

    LintOutcome {
        violations_before,
        violations_after,
        warnings_before,
        warnings_after,
        cleaned_message,
        cleanup_summaries,
    }
}

fn evaluate_message(message: &str, options: &LintOptions) -> (Vec<String>, Vec<String>) {
    let mut violations = Vec::new();
    let mut warnings = Vec::new();

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
        return (violations, warnings);
    }

    if !options.enforce_conventional_spec
        && let Some(pattern) = &options.message_pattern
        && !pattern.regex.is_match(header_line.trim())
    {
        let desc = pattern
            .description
            .as_deref()
            .unwrap_or("Commit message does not match required pattern");
        violations.push(desc.to_string());
    }

    if options.enforce_conventional_spec {
        let (mut errs, mut warns) =
            validate_conventional_commitlint_rules(message, options.body_policy);
        violations.append(&mut errs);
        warnings.append(&mut warns);
    } else {
        violations.extend(validate_body_policy(message, options.body_policy));
    }

    (violations, warnings)
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

fn apply_autofix(input: &str, enforce_conventional: bool) -> (String, Vec<String>) {
    let mut current = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut summaries = Vec::new();

    let trimmed_trailing = current
        .lines()
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n");
    if trimmed_trailing != current {
        current = trimmed_trailing;
        summaries.push("Trim trailing whitespace".to_string());
    }

    let trimmed_edges = current.trim_matches('\n').to_string();
    if trimmed_edges != current {
        current = trimmed_edges;
        summaries.push("Trim leading/trailing blank lines".to_string());
    }

    let collapsed = Regex::new("\n{3,}")
        .expect("valid regex")
        .replace_all(&current, "\n\n")
        .to_string();
    if collapsed != current {
        current = collapsed;
        summaries.push("Collapse excessive blank lines".to_string());
    }

    if enforce_conventional {
        let mut lines: Vec<&str> = current.split('\n').collect();
        if !lines.is_empty() {
            let has_content_after_header = lines.iter().skip(1).any(|line| !line.trim().is_empty());
            if has_content_after_header {
                if lines.get(1).is_some_and(|line| !line.trim().is_empty()) {
                    lines.insert(1, "");
                    summaries.push("Insert blank line before body".to_string());
                }

                if let Some(footer_start) = detect_footer_start(&lines)
                    && footer_start > 0
                    && lines
                        .get(footer_start - 1)
                        .is_some_and(|line| !line.trim().is_empty())
                {
                    lines.insert(footer_start, "");
                    summaries.push("Insert blank line before footers".to_string());
                }
            }
        }

        let rebuilt = lines.join("\n");
        if rebuilt != current {
            current = rebuilt;
        }
    }

    (current, summaries)
}

fn detect_footer_start(lines: &[&str]) -> Option<usize> {
    let mut end = lines.len();
    while end > 0 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    // Footers can span multiple lines (e.g. BREAKING CHANGE notes). Treat the footer section as the
    // suffix of the message that contains at least one recognizable footer token line.
    (0..end)
        .rev()
        .find(|&idx| parse_footer_line(lines[idx].trim_end_matches('\r')).is_some())
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

fn parse_footer_line(line: &str) -> Option<FooterEntry> {
    let line = line.trim_start();
    if line.trim().is_empty() {
        return None;
    }

    let (idx, sep_len) = if let Some(idx) = line.find(": ") {
        (idx, 2)
    } else if let Some(idx) = line.find(" #") {
        (idx, 2)
    } else {
        return None;
    };

    if idx == 0 {
        return None;
    }

    let token = line[..idx].trim().to_string();
    if token.is_empty() {
        return None;
    }

    let normalized = token.replace('-', " ");
    if !normalized.eq_ignore_ascii_case("BREAKING CHANGE") {
        // Only allow spec-shaped tokens so body text like `- Note: ...` doesn't get
        // misclassified as a footer entry.
        if !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            || token.chars().any(|c| c.is_whitespace())
        {
            return None;
        }
    }

    let value = line[(idx + sep_len)..].to_string();
    Some(FooterEntry { token, value })
}

fn validate_conventional_commitlint_rules(
    message: &str,
    policy: BodyPolicy,
) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let normalized = message.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.split('\n');
    let header = lines.next().unwrap_or("");
    let rest: Vec<&str> = lines.collect();

    let header_len = header.chars().count();
    if header_len > 100 {
        errors.push(format!(
            "header must not be longer than 100 characters, current length is {header_len}"
        ));
    }

    let header_re =
        Regex::new(r"^(\w*)(?:\((.*)\))?!?: (.*)$").expect("valid conventional header regex");
    let (ty, subject) = header_re
        .captures(header)
        .map(|caps| {
            (
                caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                caps.get(3).map(|m| m.as_str()).unwrap_or(""),
            )
        })
        .unwrap_or(("", ""));

    let allowed_types = [
        "build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style",
        "test",
    ];

    if subject.trim().is_empty() {
        errors.push("subject may not be empty".to_string());
    } else {
        let subject_trimmed = subject.trim();
        if subject_trimmed.ends_with('.') {
            errors.push("subject may not end with full stop".to_string());
        }
        if is_disallowed_subject_case(subject_trimmed) {
            errors.push(
                "subject must not be sentence-case, start-case, pascal-case, upper-case"
                    .to_string(),
            );
        }
    }

    if ty.trim().is_empty() {
        errors.push("type may not be empty".to_string());
    } else {
        if ty != ty.to_lowercase() {
            errors.push("type must be lower-case".to_string());
        }
        if !allowed_types.contains(&ty) {
            errors.push(format!(
                "type must be one of [{}]",
                allowed_types.join(", ")
            ));
        }
    }

    let (body_lines, footer_lines, footer_token_index) = split_body_and_footer(&rest);

    if policy == BodyPolicy::RequireBody {
        let body_has_content = body_lines.iter().any(|line| !line.trim().is_empty());
        if !body_has_content {
            errors.push("Commit message must include a body after a blank line".to_string());
        }
    }

    let body_has_content = body_lines.iter().any(|line| !line.trim().is_empty());
    if body_has_content && rest.first().is_some_and(|line| !line.trim().is_empty()) {
        warnings.push("body must have leading blank line".to_string());
    }

    if !footer_lines.is_empty() {
        let has_leading_blank = footer_token_index.is_some_and(|idx| {
            idx > 0 && rest.get(idx - 1).is_some_and(|line| line.trim().is_empty())
        });
        if !has_leading_blank {
            warnings.push("footer must have leading blank line".to_string());
        }
    }

    if body_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .any(|line| line.chars().count() > 100)
    {
        errors.push("body's lines must not be longer than 100 characters".to_string());
    }

    if footer_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .any(|line| line.chars().count() > 100)
    {
        errors.push("footer's lines must not be longer than 100 characters".to_string());
    }

    let footers = parse_footer_entries(&footer_lines);
    for footer in &footers {
        let token_trimmed = footer.token.trim();
        if token_trimmed.is_empty() {
            errors.push("Footer token must not be empty".to_string());
            continue;
        }

        let normalized_token = token_trimmed.replace('-', " ");
        if normalized_token.eq_ignore_ascii_case("BREAKING CHANGE") {
            if footer.token != "BREAKING CHANGE" && footer.token != "BREAKING-CHANGE" {
                errors.push(
                    "BREAKING CHANGE footer token must be uppercase (BREAKING CHANGE or BREAKING-CHANGE)"
                        .to_string(),
                );
            }
            if footer.value.trim().is_empty() {
                errors.push("BREAKING CHANGE footer must include a description".to_string());
            }
            continue;
        }

        if token_trimmed.chars().any(|c| c.is_whitespace()) {
            errors.push(format!(
                "Footer token `{}` must use hyphen in place of whitespace",
                token_trimmed
            ));
        }

        if !token_trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            errors.push(format!(
                "Footer token `{}` must use alphanumeric characters or hyphen",
                token_trimmed
            ));
        }
    }

    (errors, warnings)
}

fn split_body_and_footer<'a>(
    rest_lines: &'a [&'a str],
) -> (Vec<&'a str>, Vec<&'a str>, Option<usize>) {
    let mut end = rest_lines.len();
    while end > 0 && rest_lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    let rest_lines = &rest_lines[..end];

    let footer_start = detect_footer_start(rest_lines);
    let (body, footer) = match footer_start {
        Some(start) => (rest_lines[..start].to_vec(), rest_lines[start..].to_vec()),
        None => (rest_lines.to_vec(), Vec::new()),
    };
    (body, footer, footer_start)
}

fn parse_footer_entries(lines: &[&str]) -> Vec<FooterEntry> {
    let mut footers = Vec::new();
    let mut current: Option<FooterEntry> = None;

    for raw_line in lines {
        let line = raw_line.trim_end_matches('\r');
        if line.trim().is_empty() {
            if let Some(footer) = current.as_mut()
                && !footer.value.is_empty()
            {
                footer.value.push('\n');
            }
            continue;
        }

        if let Some(entry) = parse_footer_line(line) {
            if let Some(existing) = current.take() {
                footers.push(existing);
            }
            current = Some(entry);
            continue;
        }

        if let Some(footer) = current.as_mut() {
            if !footer.value.is_empty() {
                footer.value.push('\n');
            }
            footer.value.push_str(line);
        } else {
            return Vec::new();
        }
    }

    if let Some(existing) = current.take() {
        footers.push(existing);
    }

    footers
}

fn is_disallowed_subject_case(subject: &str) -> bool {
    is_upper_case(subject)
        || is_pascal_case(subject)
        || is_sentence_case(subject)
        || is_start_case(subject)
}

fn is_upper_case(subject: &str) -> bool {
    let mut saw_alpha = false;
    for c in subject.chars() {
        if c.is_ascii_alphabetic() {
            saw_alpha = true;
            if c.is_ascii_lowercase() {
                return false;
            }
        }
    }
    saw_alpha
}

fn is_pascal_case(subject: &str) -> bool {
    if subject.contains(char::is_whitespace) {
        return false;
    }
    let mut chars = subject.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    let mut saw_lower = false;
    let mut saw_upper = true;
    for c in chars {
        if c.is_ascii_uppercase() {
            saw_upper = true;
        } else if c.is_ascii_lowercase() {
            saw_lower = true;
        } else if !c.is_ascii_digit() && c != '_' && c != '-' {
            return false;
        }
    }
    saw_lower && saw_upper
}

fn is_sentence_case(subject: &str) -> bool {
    let words: Vec<&str> = subject.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    let first = words[0];
    let first_char = first.chars().find(|c| c.is_ascii_alphabetic());
    if first_char.is_none() || !first_char.unwrap().is_ascii_uppercase() {
        return false;
    }
    for (i, word) in words.iter().enumerate() {
        let mut word_chars = word.chars();
        let Some(start) = word_chars.find(|c| c.is_ascii_alphabetic()) else {
            continue;
        };
        if i == 0 {
            if !start.is_ascii_uppercase() {
                return false;
            }
        } else if !start.is_ascii_lowercase() {
            return false;
        }
        for c in word_chars {
            if c.is_ascii_uppercase() {
                return false;
            }
        }
    }
    true
}

fn is_start_case(subject: &str) -> bool {
    let words: Vec<&str> = subject.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    for word in words {
        let mut chars = word.chars();
        let Some(start) = chars.find(|c| c.is_ascii_alphabetic()) else {
            continue;
        };
        if !start.is_ascii_uppercase() {
            return false;
        }
        for c in chars {
            if c.is_ascii_uppercase() {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    #![allow(clippy::field_reassign_with_default)]

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
        assert!(
            outcome.warnings_before.is_empty(),
            "expected no warnings, got {:?}",
            outcome.warnings_before
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
                .warnings_before
                .iter()
                .any(|msg| msg == "body must have leading blank line"),
            "expected body-leading-blank warning"
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
                .warnings_before
                .iter()
                .any(|msg| msg == "footer must have leading blank line"),
            "expected footer-leading-blank warning"
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

    #[test]
    fn conventional_body_allows_bullets_with_colons() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>\\w+)(\\((?P<scope>.*)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "feat: add api\n\n- Update: handle edge cases\n- Note: keep API stable\n\nBREAKING CHANGE: endpoint renamed";
        let outcome = lint_message(message, &options);
        assert!(
            outcome.violations_before.is_empty(),
            "expected no violations, got {:?}",
            outcome.violations_before
        );
    }

    #[test]
    fn conventional_header_allows_digits_and_underscore() {
        let mut options = LintOptions::default();
        options.message_pattern = Some(
            build_message_pattern(
                "^(?P<type>\\w+)(\\((?P<scope>.*)\\))?(?P<breaking>!)?: (?P<description>.+)$",
                Some("Conventional".into()),
            )
            .unwrap(),
        );
        options.enforce_conventional_spec = true;
        let message = "ci(test_2): add workflow caching";
        let outcome = lint_message(message, &options);
        assert!(
            outcome.violations_before.is_empty(),
            "expected no violations, got {:?}",
            outcome.violations_before
        );
    }
}
