use log::debug;

use crate::config::AppConfig;

const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an expert software developer and git commit message writer.

Generate concise, clear commit messages following the Conventional Commits specification:
- feat: A new feature
- fix: A bug fix
- docs: Documentation only changes
- style: Changes that do not affect the meaning of the code
- refactor: A code change that neither fixes a bug nor adds a feature
- perf: A code change that improves performance
- test: Adding missing tests or correcting existing tests
- chore: Changes to the build process or auxiliary tools

Format: type(scope): description

PREFERRED FORMAT: Single line under 72 characters
Default to single line. Only use bullets for truly unrelated changes."#;

const DEFAULT_USER_PROMPT_TEMPLATE: &str = r#"Analyze the following git diff and generate a commit message.

IMPORTANT: Default to a single descriptive line under 72 characters.

Git diff:
```diff
{diff}
```

Provide only the commit message."#;

pub fn get_system_prompt() -> String {
    let a = AppConfig::load().map(|config| config.prompts.system_prompt);
    debug!("Loaded system prompt: {a:?}");
    AppConfig::load().map(|config| config.prompts.system_prompt).unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string())
}

pub fn get_user_prompt_template() -> String {
    AppConfig::load()
        .map(|config| config.prompts.user_prompt_template)
        .unwrap_or_else(|_| DEFAULT_USER_PROMPT_TEMPLATE.to_string())
}

pub fn format_commit_prompt(diff: &str) -> String {
    get_user_prompt_template().replace("{diff}", diff)
}
