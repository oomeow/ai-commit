# AI Commit Tool

[English](README.md) | [中文](README_zh.md)

一个智能的 Git 提交信息生成器，使用 AI 分析你的代码变更并自动创建有意义的、符合规范的提交信息。

## 概述

AI Commit Tool 集成到你的 Git 工作流程中，自动生成遵循 Conventional Commits 规范的高质量提交信息。它分析你的暂存或未暂存的变更，并使用 AI 来制作合适的提交信息，节省时间并确保项目的一致性。

## 功能特性

- **AI 生成提交信息**：自动分析 git diff 并生成遵循常规提交格式的上下文提交信息
- **提交前可编辑信息**：先查看 AI 结果，再用终端编辑器微调提交信息，最后确认是否提交
- **多 Provider 支持**：内置支持 OpenAI、OpenRouter、DeepSeek、Zhipu 和 Ollama
- **自动回退试运行**：当没有暂存内容时，会基于未暂存变更生成预览而不直接提交
- **修订支持**：为修订之前的提交生成新信息
- **本地消息缓存**：相同 diff 会复用已生成结果，并支持按需重新生成
- **锁文件过滤**：自动忽略常见的锁文件（Cargo.lock、package-lock.json、yarn.lock 等）的分析
- **GPG 签名支持**：与 GPG 签名的提交无缝协作
- **交互式配置初始化**：在 `config init` 中选择 provider、输入凭据并在线拉取模型
- **终端内编辑配置**：通过 `config edit` 直接使用终端编辑器修改配置
- **安全安装 Git Hook**：仅安装 `prepare-commit-msg`，且不会覆盖不属于 ai-commit 的自定义 hook
- **完全可配置**：可自定义 provider 设置、忽略模式、行为选项和 AI 提示

## 安装

### 前置要求

- Rust（最新稳定版本）
- Git 仓库
- [Just](https://github.com/casey/just) (一个方便的方式来保存和运行项目特定的命令)

### 从源码构建

```bash
git clone <repository-url>
cd ai-commit
cargo build --release
```

### 从 cargo 安装

```bash
cargo install --git https://github.com/oomeow/ai-commit.git
```

### 设置

**初始化配置**：

```bash
ai-commit config init
```

## 使用方法

### 基本命令

为暂存的变更生成提交信息：

```bash
ai-commit
# 或者明确指定
ai-commit commit
```

当存在暂存变更时，默认交互流程为：

1. AI 生成提交信息
2. 你可以选择是否在终端编辑器中打开并修改该信息
3. 再确认是否使用最终信息创建提交

先暂存当前所有变更，再生成并创建提交：

```bash
ai-commit --add
# 或者明确指定
ai-commit commit --add
```

为未暂存的变更预览提交信息（试运行模式）：

```bash
# 如果没有找到暂存的变更，将自动进入试运行模式
ai-commit
```

使用新变更修订最后一次提交：

```bash
ai-commit amend
```

### 命令选项

```bash
# 先暂存所有变更，再生成提交信息
ai-commit --add

# 仅预览生成的信息而不提交
ai-commit commit --dry-run

# 仅输出生成的提交信息
ai-commit commit --generate-only

# 将生成的提交信息写入文件
ai-commit commit --output-file .git/COMMIT_EDITMSG
```

### 配置命令

```bash
# 初始化默认配置
ai-commit config init

# 查看当前配置
ai-commit config show

# 在终端编辑器中编辑配置
ai-commit config edit
```

### Shell 补全

```bash
# 基于当前 clap 命令树生成 zsh 补全脚本
ai-commit completion zsh > _ai-commit
```

### Git 钩子集成

安装 `prepare-commit-msg` hook 以获得自动提交信息协助：

```bash
ai-commit install
```

移除已安装的 `prepare-commit-msg` hook：

```bash
ai-commit uninstall
```

### Zsh 安装

```bash
# 生成补全文件到 ~/.zsh/completions/_ai-commit
ai-commit completion zsh > ~/.zsh/completions/_ai-commit

# 将目录加入 fpath 并刷新补全
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit
compinit
```

## 配置

工具将配置存储在 `~/.config/ai-commit/config.toml` 中。使用默认设置初始化：

```bash
ai-commit config init
```

### 默认配置结构

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

### 配置选项

#### API 设置 (`[api]`)

- `provider`：内置 provider 名称，例如 `openai`、`openrouter`、`deepseek`、`zhipu`、`ollama`
- `api_key`：需要鉴权的 provider 使用的 API Key
- `base_url`：可选的基础地址覆盖项，用于拼接 chat 和 models 请求
- `endpoint`：可选的完整 chat 接口地址覆盖项
- `model`：用于生成的 AI 模型
- `max_tokens`：AI 响应的最大令牌数（默认：1000）
- `temperature`：创造性水平 0.0-1.0（默认：0.7）
- `context_limit`：为后续 diff 大小限制预留的配置项

#### 提交设置 (`[commit]`)

- `auto_confirm`：跳过确认提示（默认：false）
- `dry_run_by_default`：预留配置项，当前命令流程尚未使用
- `ignore_lock_files`：从分析中过滤出锁文件（默认：true）
- `custom_ignore_patterns`：要忽略的附加文件模式（默认：[]）

<!--#### 钩子设置 (`[hooks]`)

- `enabled`：启用 git 钩子集成（默认：false）
- `hook_types`：要安装的 git 钩子类型（默认：["prepare-commit-msg"]）-->

#### 提示设置 (`[prompts]`)

- `system_prompt`：定义 AI 行为的系统提示
- `user_prompt_template`：分析 diff 的模板（使用 `{diff}` 占位符）

### 自定义 AI 提示

你可以通过直接编辑配置文件来自定义 AI 生成提交信息的方式：

```bash
# 查看当前配置和文件位置
ai-commit config show

# 在终端编辑器中打开配置文件
ai-commit config edit
```

编辑 `~/.config/ai-commit/config.toml` 来自定义提示：

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

**自定义提示的技巧：**

- 在模板中保留 `{diff}` 占位符
- 使用 `ai-commit commit --dry-run` 测试更改
- 配置在下次运行时自动重新加载
- 在更新前备份自定义提示

### 交互式配置初始化

`ai-commit config init` 现在会进入交互式流程：

- 选择一个内置 provider
- 输入该 provider 的 API Key，或对 Ollama 这类本地 provider 留空
- 在线拉取 models，并通过分页模糊搜索选择模型
- 如果拉取失败，可回退为手动输入 model
- 其余配置项使用默认值保存

## 提交信息格式

该工具生成遵循 Conventional Commits 规范的信息：

### 单行格式（首选）

用于专注的单一目的变更：

```

feat: 添加用户认证系统
fix: 解决数据库连接超时问题
refactor: 改进认证模块的错误处理

```

### 多行格式（用于复杂变更）

当存在以下情况时使用：

1. **多个不相关的功能变更**（一次提交中的不同功能/修复）
2. **单一功能的重大变更**，需要分解说明

```

feat: 添加用户管理和通知系统

- 实现带验证的用户 CRUD 操作
- 为用户事件添加邮件通知服务
- 创建用户管理的管理员仪表板

```

### 支持的类型

- `feat`：新功能
- `fix`：错误修复
- `docs`：仅文档变更
- `style`：代码样式变更（格式化等）
- `refactor`：既不修复错误也不添加功能的代码变更
- `perf`：性能改进
- `test`：添加或修正测试
- `chore`：构建过程或辅助工具变更

## 工作流示例

### 初始设置

```bash
# 初始化配置
ai-commit config init

# 验证配置
ai-commit config show
```

### 标准工作流

```bash
# 对代码进行变更
git add .
ai-commit
# 审查生成的信息并确认
```

### 修订工作流

```bash
# 在最后一次提交后进行额外变更
git add .
ai-commit amend
# 审查合并变更的生成信息
```

### 预览变更

```bash
# 检查将生成什么信息而不提交
git add .
ai-commit commit --dry-run
```

### 自定义工作流

```bash
# 在终端编辑器中编辑配置文件
ai-commit config edit

# 测试你的更改
git add .
ai-commit commit --dry-run
```

## 技术细节

### Diff 分析

- 分析 git diff 以理解代码变更
- 自动过滤常见锁文件
- 支持自定义忽略模式
- 优先分析暂存变更，没有暂存内容时回退为未暂存预览

### AI 集成

- 使用先进的语言模型生成提交信息
- 发送上下文 diff 信息进行准确分析
- 优雅地处理 API 错误并提供回退信息
- 支持不同提交风格的完全可自定义提示
- 可在配置阶段从所选 provider 在线拉取 models

### 配置管理

- 符合 XDG Base Directory 规范（`~/.config/ai-commit/`）
- TOML 格式便于编辑和版本控制
- 配置缺失时会自动创建默认配置
- 通过 `config edit` 保存前会校验 TOML 格式
- 配置变更的热重载，无需重启
- 支持 provider 级别的 `base_url` 和 `endpoint` 覆盖

### 安全性

- 与 GPG 签名的提交协作
- 遵守 git 配置设置
- 不在外部存储代码或敏感信息
- 具有适当权限的本地配置文件
- 不会覆盖非 `ai-commit` 创建的 `prepare-commit-msg` hook

## 贡献

欢迎贡献！请随时：

- 通过 issues 提交错误报告和功能请求
- 为改进创建拉取请求
- 分享反馈和建议
- 帮助改进文档

### 开发设置

```bash
git clone <repository-url>
cd ai-commit
cargo test

# 调试命令, 输出 debug 日志
# 例如:
#   ai-commit --add -> just dev --add
#   ai-commit config show -> just dev config show
just dev [命令]

# 测试命令, 没有日志输出
# 例如:
#   ai-commit --add -> just test --add
#   ai-commit config show -> just test config show
just test [命令]
```

## 许可证

该项目采用 MIT 许可证。详见 LICENSE 文件。
