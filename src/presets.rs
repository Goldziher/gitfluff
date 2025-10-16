use crate::lint::BodyPolicy;

#[derive(Debug, Clone)]
pub struct Preset {
    pub message_pattern: &'static str,
    pub description: &'static str,
    pub excludes: &'static [PresetExclude],
    pub cleanup: &'static [PresetCleanup],
    pub body_policy: BodyPolicy,
}

#[derive(Debug, Clone)]
pub struct PresetExclude {
    pub pattern: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone)]
pub struct PresetCleanup {
    pub find: &'static str,
    pub replace: &'static str,
    pub description: &'static str,
}

const CONVENTIONAL_PATTERN: &str = "^(?<type>build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\\([^)]+\\))?!?: (?<description>.+)$";
const ATOM_PATTERN: &str = "^(?<type>build|chore|ci|docs|feat|fix|perf|refactor|style|test)(\\([^)]+\\))?!?: (?<description>.+)$";
const EMBER_PATTERN: &str = "^(?<type>build|chore|ci|docs|feat|fix|perf|refactor|style|test|breaking)(\\([^)]+\\))?!?: (?<description>.+)$";
const ESLINT_PATTERN: &str = "^(?<type>build|chore|docs|feat|fix|perf|refactor|test|update)(\\([^)]+\\))?!?: (?<description>.+)$";
const EXPRESS_PATTERN: &str = "^(?<type>build|chore|ci|docs|feat|fix|perf|refactor|test)(\\([^)]+\\))?!?: (?<description>.+)$";
const JSHINT_PATTERN: &str = "^(?<type>build|chore|ci|docs|feat|fix|perf|refactor|test)(\\([^)]+\\))?!?: (?<description>.+)$";

pub fn resolve_preset(name: &str) -> Option<Preset> {
    match name.to_lowercase().as_str() {
        "conventional" => Some(conventional()),
        "angular" => Some(angular()),
        "conventional-body" | "conventional_detailed" | "conventional-with-body" => {
            Some(conventional_with_body())
        }
        "atom" => Some(atom()),
        "ember" => Some(ember()),
        "eslint" => Some(eslint()),
        "express" => Some(express()),
        "no-ai" | "conventional-no-ai" | "conventional_ai_safe" => Some(conventional_no_ai()),
        "gitmoji" => Some(gitmoji()),
        "jshint" => Some(jshint()),
        "simple" | "simple-single-line" => Some(simple_single_line()),
        _ => None,
    }
}

fn conventional() -> Preset {
    Preset {
        message_pattern: CONVENTIONAL_PATTERN,
        description: "Use Conventional Commits format: type[(scope)]!: description",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn angular() -> Preset {
    Preset {
        message_pattern: CONVENTIONAL_PATTERN,
        description: "Use Angular-style Conventional Commits format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn atom() -> Preset {
    Preset {
        message_pattern: ATOM_PATTERN,
        description: "Use Atom commit convention format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn ember() -> Preset {
    Preset {
        message_pattern: EMBER_PATTERN,
        description: "Use Ember commit convention format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn eslint() -> Preset {
    Preset {
        message_pattern: ESLINT_PATTERN,
        description: "Use ESLint commit convention format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn express() -> Preset {
    Preset {
        message_pattern: EXPRESS_PATTERN,
        description: "Use Express.js commit convention format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn conventional_with_body() -> Preset {
    Preset {
        message_pattern: CONVENTIONAL_PATTERN,
        description: "Use Conventional Commits format with a required body section",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::RequireBody,
    }
}

fn conventional_no_ai() -> Preset {
    static EXCLUDES: &[PresetExclude] = &[
        PresetExclude {
            pattern: "(?mi)^Co-Authored-By:.*(?:Claude|Anthropic|ChatGPT|GPT|OpenAI).*$",
            message: "Remove AI co-author attribution lines",
        },
        PresetExclude {
            pattern: "🤖 Generated with",
            message: "Remove AI generation notices from commit messages",
        },
    ];

    static CLEANUPS: &[PresetCleanup] = &[
        PresetCleanup {
            find: "(?m)^.*🤖 Generated with.*\n?",
            replace: "",
            description: "Remove AI generation banner",
        },
        PresetCleanup {
            find: "(?mi)^Co-Authored-By:.*\n?",
            replace: "",
            description: "Drop Co-Authored-By lines referencing AI assistants",
        },
        PresetCleanup {
            find: "(?s)\\A\\s*\n+",
            replace: "",
            description: "Trim leading blank lines",
        },
        PresetCleanup {
            find: "\n{3,}",
            replace: "\n\n",
            description: "Collapse excessive blank lines",
        },
        PresetCleanup {
            find: "(?s)\n\\s*\n\\z",
            replace: "\n",
            description: "Trim trailing blank lines",
        },
    ];

    Preset {
        message_pattern: CONVENTIONAL_PATTERN,
        description: "Use Conventional Commits format and strip AI attribution lines",
        excludes: EXCLUDES,
        cleanup: CLEANUPS,
        body_policy: BodyPolicy::Any,
    }
}

fn gitmoji() -> Preset {
    const GITMOJI_PATTERN: &str =
        "^:[a-z0-9_+\\-]+:(?:\\s\\([^)]+\\))?(?:\\s#[0-9]+|\\shttps?://\\S+|\\s\\[[^]]+\\])?\\s.+$";
    Preset {
        message_pattern: GITMOJI_PATTERN,
        description: "Start with a gitmoji like :sparkles: followed by a summary",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn jshint() -> Preset {
    Preset {
        message_pattern: JSHINT_PATTERN,
        description: "Use JSHint commit convention format",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::Any,
    }
}

fn simple_single_line() -> Preset {
    const SIMPLE_PATTERN: &str = "^[A-Z][^\\n]+$";
    Preset {
        message_pattern: SIMPLE_PATTERN,
        description: "Single-line summary starting with a capital letter",
        excludes: &[],
        cleanup: &[],
        body_policy: BodyPolicy::SingleLine,
    }
}
