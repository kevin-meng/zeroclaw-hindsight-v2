# ZeroClaw + Hindsight Long-Term Memory

[English](#english) · [中文](#中文)

---

## English

> **ZeroClaw** is a Rust-first autonomous AI agent runtime — fast, small, and extensible.
> This fork adds **Hindsight** long-term memory integration, giving the agent persistent semantic memory with knowledge graph and cross-memory reasoning.

### What is Hindsight?

[Hindsight](https://github.com/vectorize-io/hindsight) is a cloud-native long-term memory service for AI agents. It provides:

- **Semantic Search** — Store and retrieve memories using natural language
- **Knowledge Graph** — Entity resolution and relationship tracking across memories
- **Cross-Memory Reasoning** — Synthesize insights from multiple related memories
- **Multi-Strategy Retrieval** — adaptive recall based on query context and budget

### Features

| Feature | Description |
|---------|-------------|
| `hindsight_retain` | Store information to long-term memory with context and tags |
| `hindsight_recall` | Semantic search across all stored memories |
| `hindsight_reflect` | Reason across memories to synthesize a coherent answer |
| Configurable Budget | `low` / `mid` / `high` recall depth control |
| Auto-retain | Optional automatic memory capture on each conversation turn |

### Quick Start

#### 1. Configure ZeroClaw

```toml
# config.toml

[memory]
backend = "hindsight"

[memory.hindsight]
api_url = "https://api.hindsight.vectorize.io"
api_key = "${HINDSIGHT_API_KEY}"   # Set env var or use literal key
bank_id = "zeroclaw"               # Your memory bank identifier
budget = "mid"                     # low | mid | high
timeout_secs = 120
```

#### 2. Environment Variable

```bash
export HINDSIGHT_API_KEY="your-api-key-here"
```

#### 3. Run

```bash
cargo run --release
```

### Configuration Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `api_url` | `string` | `https://api.hindsight.vectorize.io` | Hindsight API endpoint |
| `api_key` | `string` | `${HINDSIGHT_API_KEY}` | API key (or env var) |
| `bank_id` | `string` | `zeroclaw` | Memory bank identifier |
| `budget` | `string` | `mid` | Recall budget: `low`, `mid`, or `high` |
| `timeout_secs` | `u64` | `120` | Request timeout in seconds |
| `auto_retain` | `bool` | `true` | Automatically retain each turn |
| `auto_recall` | `bool` | `true` | Automatically recall relevant memories before each turn |
| `retain_tags` | `[]string` | `[]` | Tags attached to all retained memories |
| `retain_source` | `string` | `null` | Source label for retained memories |

### Architecture

```
zeroclaw-runtime
  └── tools/mod.rs          # Tool registration
       ├── hindsight_retain.rs   # retain tool
       ├── hindsight_recall.rs  # recall tool
       └── hindsight_reflect.rs # reflect tool

zeroclaw-tools
  ├── hindsight.rs         # Shared types (Budget, RecallResult, ...)
  ├── hindsight_client.rs  # REST API client
  └── hindsight_config.rs  # HindsightConfig (zeroclaw-tools layer)

zeroclaw-config
  └── schema.rs            # HindsightSchemaConfig (config layer)
```

### Build

```bash
# Install Rust 1.93+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Check compilation
cargo check --workspace

# Run tests
cargo test

# Build release
cargo build --release
```

### Related Projects

- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — The base agent runtime
- [Hindsight](https://github.com/vectorize-io/hindsight) — Long-term memory backend
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — Reference implementation

### License

MIT OR Apache-2.0

---

## 中文

> **ZeroClaw** 是一个 Rust-first 的自主 AI Agent 运行时——快、小巧、可扩展。
> 本分支集成了 **Hindsight** 长期记忆系统，为 Agent 提供持久化语义记忆、知识图谱和跨记忆推理能力。

### 什么是 Hindsight？

[Hindsight](https://github.com/vectorize-io/hindsight) 是面向 AI Agent 的云端长期记忆服务，提供：

- **语义搜索** — 用自然语言存储和检索记忆
- **知识图谱** — 跨记忆的实体解析和关系追踪
- **跨记忆推理** — 从多个相关记忆中综合提炼洞察
- **多策略检索** — 根据查询上下文和预算自适应召回

### 功能一览

| 功能 | 说明 |
|------|------|
| `hindsight_retain` | 将信息存入长期记忆，支持上下文和标签 |
| `hindsight_recall` | 在所有存储记忆中语义搜索 |
| `hindsight_reflect` | 跨记忆推理，综合连贯答案 |
| 可配置预算 | `low` / `mid` / `high` 三档召回深度 |
| 自动记忆 | 可选：每次对话轮次自动捕获记忆 |

### 快速开始

#### 1. 配置 ZeroClaw

```toml
# config.toml

[memory]
backend = "hindsight"

[memory.hindsight]
api_url = "https://api.hindsight.vectorize.io"
api_key = "${HINDSIGHT_API_KEY}"   # 设置环境变量或直接写 key
bank_id = "zeroclaw"               # 你的记忆库标识符
budget = "mid"                     # low | mid | high
timeout_secs = 120
```

#### 2. 设置环境变量

```bash
export HINDSIGHT_API_KEY="your-api-key-here"
```

#### 3. 运行

```bash
cargo run --release
```

### 配置参考

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `api_url` | `string` | `https://api.hindsight.vectorize.io` | Hindsight API 地址 |
| `api_key` | `string` | `${HINDSIGHT_API_KEY}` | API 密钥 |
| `bank_id` | `string` | `zeroclaw` | 记忆库标识符 |
| `budget` | `string` | `mid` | 召回预算：`low`、`mid` 或 `high` |
| `timeout_secs` | `u64` | `120` | 请求超时（秒） |
| `auto_retain` | `bool` | `true` | 自动保留每轮对话 |
| `auto_recall` | `bool` | `true` | 每轮对话前自动召回相关记忆 |
| `retain_tags` | `[]string` | `[]` | 附加到所有记忆的标签 |
| `retain_source` | `string` | `null` | 记忆来源标签 |

### 项目结构

```
zeroclaw-runtime
  └── tools/mod.rs          # 工具注册
       ├── hindsight_retain.rs   # retain 工具
       ├── hindsight_recall.rs  # recall 工具
       └── hindsight_reflect.rs # reflect 工具

zeroclaw-tools
  ├── hindsight.rs         # 共享类型（Budget, RecallResult, …）
  ├── hindsight_client.rs  # REST API 客户端
  └── hindsight_config.rs  # HindsightConfig

zeroclaw-config
  └── schema.rs            # HindsightSchemaConfig（配置层）
```

### 构建

```bash
# 安装 Rust 1.93+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 检查编译
cargo check --workspace

# 运行测试
cargo test

# Release 构建
cargo build --release
```

### 相关项目

- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) — 基础 Agent 运行时
- [Hindsight](https://github.com/vectorize-io/hindsight) — 长期记忆后端
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — 参考实现

### 开源许可

MIT OR Apache-2.0