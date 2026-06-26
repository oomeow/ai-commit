# 多 Provider 配置 + 默认 Provider 选择

## 目标
- 配置文件支持配置多个 provider（每个有独立的 name / api_key / model / endpoint 等）。
- 配置文件中指定一个默认 provider。
- CLI 支持通过 `-p/--provider <name>` 临时选择本次运行使用的 provider。
- 新增 `config provider` 子命令组管理多 provider（list / use / add / remove）。
- 完全向后兼容：旧的单 `[api]` 配置仍可加载，自动迁移为单条 provider。

## 配置文件新格式 (config.sample.toml)
```toml
default_provider = "openrouter"

[[providers]]
name = "openrouter"           # 用户自定义标识，default_provider 引用此值
provider = "openrouter"       # 内置 provider 类型（决定 base_url / protocol）
api_key = "..."
model = "z-ai/glm-4.5-air:free"
max_tokens = 1000
temperature = 0.7

[[providers]]
name = "local"
provider = "ollama"
model = "qwen2.5:14b"

[commit]
...
[prompts]
...
```

旧格式（单 `[api]` 表，无 `providers`）仍能解析并自动作为唯一/默认 provider。

## 代码改动

### 1. `src/config/settings.rs`
- `ApiConfig` 增加字段 `pub name: Option<String>`（serde `default`，TOML 中的条目标识）。
- `AppConfig`:
  - `api` 改为 `#[serde(default)] pub api: Option<ApiConfig>`（兼容旧格式）。
  - 新增 `#[serde(default)] pub default_provider: Option<String>`。
  - 新增 `#[serde(default)] pub providers: Vec<ApiConfig>`。
- `load()` / `load_from_path()` 加载后调用 `normalize()`：若 `providers` 为空且 `api` 有值，则把 `api` 迁入 `providers`（name 取 `name` -> `provider` -> "default"）。
- 新增方法：
  - `active_provider(&self, override_name: Option<&str>) -> Result<&ApiConfig>`：按 override -> default_provider -> 第一个 解析；找不到指定名报错。
  - `provider_names(&self) -> Vec<&str>`。
- `save()` 序列化时清掉已迁移的旧 `api`（设为 None），只写新格式。

### 2. `src/ai/client.rs`
- `AiClient` 增加 `provider_override: Option<String>` 字段。
- 新增 `with_provider(mut self, name: Option<String>) -> Self` builder。
- `fetch_provider_models` / `send_chat_request` 改用 `self.config.active_provider(self.provider_override.as_deref())?` 替代 `&self.config.api`。
- `AppConfig::generate_user_prompt` 不变（prompts 仍全局共享）。

### 3. `src/ai/provider.rs`
- `resolve_api_config` 签名不变（仍接收 `&ApiConfig`）。更新单测里构造 `ApiConfig` 时补上 `name: None`。

### 4. `src/main.rs`
- 顶层与 `commit`/`amend` 子命令增加 `-p/--provider <NAME>` 参数。
- 解析出 `provider` 透传给 `execute_command`。

### 5. `src/commands/mod.rs`
- `execute_command` 增加 `provider: Option<&str>` 形参，传入 commit/amend handler。
- 新增 `config-provider-list` / `config-provider-use` / `config-provider-add` / `config-provider-remove` 分发。

### 6. `src/commands/commit.rs` & `amend.rs`
- handler 增加 provider 参数，构造 client 时 `.with_provider(...)`。

### 7. `src/commands/config.rs`
- `init_config` 改为「添加/更新一个命名 provider 条目」：先输入条目 name，选内置 provider，输入 key、选 model，写入 `providers`；若是首条则设为 `default_provider`。
- 新增：
  - `list_providers`：列出所有 provider，标记默认项。
  - `use_provider(name)`：设置 `default_provider`。
  - `add_provider` / `remove_provider`：增删条目（删除默认项时重置 default）。

### 8. 测试
- `settings.rs` 增加单测：旧格式迁移、多 provider 解析、active_provider 选择逻辑（override / default / fallback / 错误名）。
- 更新 `provider.rs` 现有单测的 `ApiConfig` 构造。
- 运行 `cargo build` + `cargo test` 验证。

## 命名说明
- `name` = 用户给配置条目起的标识（如 "work-openrouter"、"local"）。
- `provider` = 内置 provider 类型（openai/ollama/...），保持原语义。
- `default_provider` 引用 `name`。
