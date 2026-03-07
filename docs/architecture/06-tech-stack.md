# 技术栈


---

## 6. 技术栈

### 6.1 核心依赖

#### 6.1.1 异步运行时与网络

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tokio` | 1.x | 异步运行时 |
| `futures` | 0.3 | Future 和 Stream 抽象 |
| `async-trait` | 0.1 | 异步 trait 支持 |

#### 6.1.2 Web 框架

| 依赖 | 版本 | 用途 |
|------|------|------|
| `axum` | 0.8 | HTTP 框架 |
| `tonic` | 0.12 | gRPC 框架 |
| `tower` | 0.5 | 中间件抽象 |
| `tower-http` | 0.6 | HTTP 中间件 |

#### 6.1.3 共识算法

| 依赖 | 版本 | 用途 |
|------|------|------|
| `async-raft` | 0.6 | Raft 实现（Tokio 原生） |

#### 6.1.4 存储引擎

| 依赖 | 版本 | 用途 |
|------|------|------|
| `rocksdb` | 0.22 | RocksDB 绑定（可选） |
| `sled` | 0.34 | 嵌入式数据库（可选） |

#### 6.1.5 序列化

| 依赖 | 版本 | 用途 |
|------|------|------|
| `serde` | 1.x | 序列化框架 |
| `serde_json` | 1.x | JSON 序列化 |
| `bincode` | 1.x | 二进制序列化（高性能） |
| `prost` | 0.13 | Protocol Buffers |

#### 6.1.6 错误处理与日志

| 依赖 | 版本 | 用途 |
|------|------|------|
| `thiserror` | 2.x | 自定义错误类型 |
| `anyhow` | 1.x | 错误处理 |
| `tracing` | 0.1 | 结构化日志 |
| `tracing-subscriber` | 0.3 | 日志订阅器 |

### 6.2 开发工具链

#### 6.2.1 构建工具

- **Cargo**: Rust 包管理和构建工具
- **cargo-make**: 任务运行器（可选）

#### 6.2.2 代码生成

- **tonic-build**: gRPC 代码生成
- **prost-build**: Protobuf 代码生成

#### 6.2.3 测试工具

- **cargo-test**: 单元测试和集成测试
- **criterion**: 性能基准测试
- **tokio-test**: Tokio 测试工具

#### 6.2.4 代码质量

- **clippy**: Linter
- **rustfmt**: 代码格式化
- **cargo-audit**: 安全审计

### 6.3 监控与可观测性

#### 6.3.1 指标收集

| 指标类型 | 示例 |
|----------|------|
| **性能指标** | QPS, 延迟 (P50/P95/P99), 吞吐量 |
| **Raft 指标** | Leader 选举次数, 日志复制延迟, 快照大小 |
| **存储指标** | 存储大小, 读写次数, 缓存命中率 |
| **系统指标** | CPU, 内存, 磁盘 I/O, 网络流量 |

#### 6.3.2 日志

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
    info!(key_len = key.len(), value_len = value.len(), "Processing PUT");
    // ...
    info!("PUT completed successfully");
    Ok(())
}
```

#### 6.3.3 分布式追踪

- 使用 `tracing-opentelemetry` 集成 OpenTelemetry
- 支持跨服务的请求追踪

---


---

