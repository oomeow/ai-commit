# AI Commit Tool

[English](README.md) | [中文](README_zh.md)

An intelligent Git commit message generator that uses AI to analyze your code changes and create meaningful, conventional commit messages automatically.

## Overview

AI Commit Tool integrates with your Git workflow to automatically generate high-quality commit messages following the Conventional Commits specification. It analyzes your staged or unstaged changes and uses AI to craft appropriate commit messages, saving time and ensuring consistency across your project.

## Features

- **AI-Generated Commit Messages**: Automatically analyzes git diffs and generates contextual commit messages following conventional commit format
- **Edit Before Confirming**: Review the AI result, open it in your terminal editor, and fine-tune the message before the final commit prompt
- **Multiple Provider Support**: Built-in support for OpenAI, OpenRouter, DeepSeek, Zhipu, and Ollama
- **Automatic Dry-Run Fallback**: If nothing is staged, the tool previews a message from unstaged changes instead of committing
- **Amend Support**: Generate new messages for amending previous commits with additional changes
- **Local Message Cache**: Reuses generated messages for the same diff and lets you regenerate on demand
- **Lock File Filtering**: Automatically ignores common lock files (Cargo.lock, package-lock.json, yarn.lock, etc.) from analysis
- **GPG Signing Support**: Works seamlessly with GPG-signed commits
- **Interactive Config Setup**: Choose a provider, enter credentials, and fetch models during `config init`
- **Editable Config in Terminal**: Open the config file with your terminal editor via `config edit`
- **Safe Hook Installation**: Installs only the `prepare-commit-msg` hook and avoids overwriting unrelated custom hooks
- **Fully Configurable**: Customizable provider settings, ignore patterns, behavior options, and AI prompts

## Installation

### Prerequisites

- Rust (latest stable version)
- Git repository
- [Just](https://github.com/casey/just) (a handy way to save and run project-specific commands)

### Build from Source

```bash
git clone <repository-url>
cd ai-commit
cargo build --release
```

### Install from cargo

```bash
cargo install --git https://github.com/oomeow/ai-commit.git
```

### Setup

**Initialize Configuration**:

```bash
ai-commit config init
```

## Usage

### Basic Commands

Generate commit message for staged changes:

```bash
ai-commit
# or explicitly
ai-commit commit
```

Default interactive flow for staged changes:

1. AI generates a commit message
2. You can choose whether to open the message in your terminal editor
3. You confirm whether to create the commit with the final message

Stage all current changes first, then generate and create the commit:

```bash
ai-commit --add
# or explicitly
ai-commit commit --add
```

Preview commit message for unstaged changes (dry-run mode):

```bash
# Will automatically enter dry-run mode if no staged changes found
ai-commit
```

Amend the last commit with new changes:

```bash
ai-commit amend
```

### Command Options

```bash
# Stage all changes before generating the commit message
ai-commit --add

# Preview a generated message without committing
ai-commit commit --dry-run

# Print only the generated message
ai-commit commit --generate-only

# Write the generated message to a file
ai-commit commit --output-file .git/COMMIT_EDITMSG
```

### Configuration Commands

```bash
# Initialize default configuration
ai-commit config init

# View current configuration
ai-commit config show

# Edit configuration in your terminal editor
ai-commit config edit
```

### Shell Completion

```bash
# Generate zsh completion from the current clap command tree
ai-commit completion zsh > _ai-commit
```

### Git Hooks Integration

Install the `prepare-commit-msg` hook for automatic commit message assistance:

```bash
ai-commit install
```

Remove the installed `prepare-commit-msg` hook:

```bash
ai-commit uninstall
```

### Zsh Installation

```bash
# Generate the completion file
ai-commit completion zsh > ~/.zsh/completions/_ai-commit

# Load the directory from your fpath, then refresh completions
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit
compinit
```

## Configuration

The tool stores configuration in `~/.config/ai-commit/config.toml`. Initialize with default settings:

```bash
ai-commit config init
```

### Default Configuration Structure

```toml
[api]
provider = "openrouter"
api_key = "12345-678910-1122-3344-123123123123"
model = "z-ai/glm-4.5-air:free"
max_tokens = 1000
temperature = 0.7
context_limit = 200000

[commit]
auto_confirm = false
dry_run_by_default = false
ignore_lock_files = true
custom_ignore_patterns = []

# [hooks]
# enabled = false
# hook_types = ["prepare-commit-msg"]

[prompts]
system_prompt = """You are a senior software engineer writing precise Git commit messages..."""
user_prompt_template = """Review the following Git diff and write the best commit message..."""
```

### Configuration Options

#### API Settings (`[api]`)

- `provider`: Built-in provider name such as `openai`, `openrouter`, `deepseek`, `zhipu`, or `ollama`
- `api_key`: API key used for authenticated providers
- `base_url`: Optional base URL override used to build chat and models requests
- `endpoint`: Optional full chat endpoint override
- `model`: AI model to use for generation
- `max_tokens`: Maximum tokens for AI response (default: 1000)
- `temperature`: Creativity level 0.0-1.0 (default: 0.7)
- `context_limit`: Reserved configuration for future diff-size limiting

#### Commit Settings (`[commit]`)

- `auto_confirm`: Skip confirmation prompt (default: false)
- `dry_run_by_default`: Reserved configuration field, not currently applied by the command flow
- `ignore_lock_files`: Filter out lock files from analysis (default: true)
- `custom_ignore_patterns`: Additional file patterns to ignore (default: [])

<!--#### Hook Settings (`[hooks]`)

- `enabled`: Enable git hooks integration (default: false)
- `hook_types`: Types of git hooks to install (default: ["prepare-commit-msg"])-->

#### Prompt Settings (`[prompts]`)

- `system_prompt`: System prompt that defines AI behavior
- `user_prompt_template`: Template for analyzing diffs (use `{diff}` placeholder)

### Customizing AI Prompts

You can fully customize how the AI generates commit messages by editing the configuration file directly:

```bash
# View current configuration and file location
ai-commit config show

# Open the config file in your terminal editor
ai-commit config edit
```

Edit `~/.config/ai-commit/config.toml` to customize prompts:

````toml
[prompts]
system_prompt = """You are a senior software engineer writing precise Git commit messages.
You MUST follow Conventional Commits.
Output only the final commit message."""

user_prompt_template = """Review the following Git diff and write the best commit message.
Return only the commit message.
Prefer a single-line Conventional Commit.

Git diff:
```diff
{diff}
```"""

````

**Tips for Custom Prompts:**

- Keep the `{diff}` placeholder in templates
- Test changes with `ai-commit commit --dry-run`
- Configuration reloads automatically on next run
- Back up custom prompts before updates

### Interactive Configuration

`ai-commit config init` now runs as an interactive setup flow:

- Select one of the built-in providers
- Enter the provider API key, or leave it empty for local providers such as Ollama
- Fetch models online and choose one with paginated fuzzy search
- Fall back to manual model input if model fetching fails
- Save the rest of the settings with sensible defaults

## Commit Message Format

The tool generates messages following the Conventional Commits specification:

### Single-line Format (preferred)

Used for focused changes with single purpose:

```

feat: add user authentication system
fix: resolve database connection timeout
refactor: improve error handling in auth module

```

### Multi-line Format (for complex changes)

Used when there are:

1. **Multiple unrelated functional changes** (different features/fixes in one commit)
2. **Single feature with significant changes** that benefit from breakdown explanation

```

feat: add user management and notification system

- Implement user CRUD operations with validation
- Add email notification service for user events
- Create admin dashboard for user management

```

### Supported Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code changes that neither fix bugs nor add features
- `perf`: Performance improvements
- `test`: Adding or correcting tests
- `chore`: Build process or auxiliary tool changes

## Workflow Examples

### Initial Setup

```bash
# One-time setup
export AI_COMMIT_ARK_API_KEY="your-api-key"
ai-commit config init

# Verify configuration
ai-commit config show
```

### Standard Workflow

```bash
# Make changes to your code
git add .
ai-commit
# Review generated message and confirm
```

### Amend Workflow

```bash
# Make additional changes after last commit
git add .
ai-commit amend
# Review generated message for combined changes
```

### Preview Changes

```bash
# Check what message would be generated without committing
git add .
ai-commit commit --dry-run
```

### Customization Workflow

```bash
# Edit configuration file in your terminal editor
ai-commit config edit

# Test your changes
git add .
ai-commit commit --dry-run
```

## Technical Details

### Diff Analysis

- Analyzes git diffs to understand code changes
- Filters out common lock files automatically
- Supports custom ignore patterns
- Prefers staged changes and falls back to unstaged changes in preview mode

### AI Integration

- Uses advanced language models for commit message generation
- Sends contextual diff information for accurate analysis
- Handles API errors gracefully with fallback messages
- Supports fully customizable prompts for different commit styles
- Can fetch available models from the selected provider during configuration

### Configuration Management

- XDG Base Directory specification compliant (`~/.config/ai-commit/`)
- TOML format for easy editing and version control
- Creates a default config automatically when missing
- Validates edited TOML before saving through `config edit`
- Hot-reload of configuration changes without restart
- Supports provider-level `base_url` and `endpoint` overrides

### Security

- Works with GPG-signed commits
- Respects git configuration settings
- No code or sensitive information stored externally
- Local configuration files with proper permissions
- Will not overwrite non-`ai-commit` `prepare-commit-msg` hooks

## Contributing

Contributions are welcome! Please feel free to:

- Submit bug reports and feature requests through issues
- Create pull requests for improvements
- Share feedback and suggestions
- Help improve documentation

### Development Setup

```bash
git clone <repository-url>
cd ai-commit
cargo test

# debug commands, output debug logs
# such as:
#   ai-commit --add -> just dev --add
#   ai-commit config show -> just dev config show
just dev [commands]

# test commands, no log output
# such as:
#   ai-commit --add -> just test --add
#   ai-commit config show -> just test config show
just test [commands]
```

## License

This project is licensed under the MIT License. See the LICENSE file for details.
