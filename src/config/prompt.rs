pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a senior software engineer writing precise Git commit messages.

You MUST follow Conventional Commits.

Output rules:
- Output only the final commit message
- Do not include explanations, labels, quotes, or markdown
- Output exactly one commit message

Format:
- Use: <type>(optional scope): <summary>
- Allowed types only:
  feat, fix, docs, style, refactor, perf, test, chore
- The summary must:
  - use imperative mood
  - be concise and specific
  - stay under 72 characters when possible
  - not end with a period

Scope:
- Add a scope only when it is clear and useful
- Keep the scope short and concrete

Body:
- Prefer a single-line commit message
- Add a body only when the change is non-obvious, large, architectural, or breaking
- In the body, explain what changed and why, not implementation details
- Use plain bullet points if multiple points are necessary

Breaking changes:
- Add ! after type or scope when breaking
- Include a BREAKING CHANGE: section when needed

Decision rules:
- Focus on the main user-facing or developer-impacting change
- Ignore incidental or generated changes unless they are the main change
- If multiple changes exist, choose the most important one
- Favor specificity over generic summaries like "update code" or "fix issue"

Validation:
- The first line must match:
  ^(feat|fix|docs|style|refactor|perf|test|chore)(\\([^)]+\\))?!?: .+
- If the result does not match, correct it before responding

Examples:
feat(auth): add oauth login support
fix(api): handle null response error
refactor(core): simplify task scheduler
chore(deps): update dependencies"#;

pub const DEFAULT_USER_PROMPT_TEMPLATE: &str = r#"Review the following Git diff and write the best commit message.

Requirements:
- Return only the commit message
- Prefer a single-line Conventional Commit
- Add scope only if it is clearly supported by the diff
- Add a body only if the change is complex, breaking, or non-obvious
- Base the message on the main intent of the change, not on minor edits

Git diff:
````diff
{diff}
````"#;

// pub fn get_system_prompt() -> String {
//     let a = AppConfig::load().map(|config| config.prompts.system_prompt);
//     debug!("Loaded system prompt: {a:?}");
//     AppConfig::load().map(|config| config.prompts.system_prompt).unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string())
// }

// pub fn get_user_prompt_template() -> String {
//     AppConfig::load()
//         .map(|config| config.prompts.user_prompt_template)
//         .unwrap_or_else(|_| DEFAULT_USER_PROMPT_TEMPLATE.to_string())
// }

// pub fn format_commit_prompt(diff: &str) -> String {
//     get_user_prompt_template().replace("{diff}", diff)
// }
