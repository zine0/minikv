# MiniKV 分布式键值存储架构设计文档

**版本**: v1.0  
**日期**: 2026-03-07  
**状态**: 设计阶段

---

## 目录

1. [概述](#1-概述)
2. [系统架构](#2-系统架构)
3. [模块设计](#3-模块设计)
4. [核心接口设计](#4-核心接口设计)
5. [数据流设计](#5-数据流设计)
6. [技术栈](#6-技术栈)
7. [部署与运维](#7-部署与运维)
8. [实施路线图](#8-实施路线图)
9. [附录](#9-附录)

---

## 1. 概述

### 1.1 项目目标

MiniKV 是一个基于 Rust 和 Tokio 构建的分布式键值存储系统，旨在提供：

- **强一致性保证**: 通过 Raft 共识算法实现分布式一致性
- **灵活的部署模式**: 支持单节点和集群两种部署模式
- **可插拔的存储引擎**: 通过 trait 抽象支持多种存储后端
- **多协议 API 支持**: 同时提供 HTTP RESTful 和 gRPC 接口

### 1.2 设计原则

#### 1.2.1 一致性优先

在 CAP 理论的权衡中，MiniKV 选择 **CP (Consistency + Partition Tolerance)** 模型：

- **强一致性**: 通过 Raft 协议保证所有节点的数据一致性
- **分区容错**: 在网络分区发生时，优先保证数据一致性而非可用性
- **牺牲可用性**: 在网络分区时，少数派分区将无法提供服务

#### 1.2.2 模块化设计

- **关注点分离**: 每个模块职责单一，接口清晰
- **可测试性**: 通过 trait 抽象实现依赖注入，便于单元测试
- **可扩展性**: 存储引擎和共识算法可独立扩展和替换

#### 1.2.3 Rust 最佳实践

- **类型安全**: 充分利用 Rust 的类型系统防止运行时错误
- **零成本抽象**: trait 对象使用泛型而非 `dyn Trait`，避免性能损失
- **异步优先**: 所有 I/O 操作基于 Tokio 异步运行时

### 1.3 核心特性

| 特性 | 描述 |
|------|------|
| **强一致性** | 基于 Raft 的分布式共识，保证线性一致性 |
| **双部署模式** | 单节点模式（开发测试）+ 集群模式（生产环境） |
| **可插拔存储** | In-Memory、自定义磁盘存储、Sled（未来支持） |
| **多协议 API** | HTTP RESTful + gRPC 双接口 |
| **高性能异步** | 基于 Tokio 的异步 I/O，支持高并发 |
| **配置驱动** | 通过配置文件切换部署模式和存储后端 |

### 1.4 非功能性需求

#### 1.4.1 性能目标

| 指标 | 单节点模式 | 集群模式 (3节点) | 参考 |
|------|-----------|----------------|------|
| **读吞吐量** | 50k+ ops/s | 20k+ ops/s | etcd: ~30k ops/s |
| **写吞吐量** | 30k+ ops/s | 10k+ ops/s | etcd: ~10k ops/s |
| **读延迟 (P99)** | < 1ms | < 5ms | - |
| **写延迟 (P99)** | < 2ms | < 10ms | - |

#### 1.4.2 可用性目标

- **单节点可用性**: 99.9% (依赖底层存储持久化)
- **集群可用性**: 99.99% (容忍 1 个节点故障)
- **故障恢复时间**: < 30s (Raft 选举超时配置)

#### 1.4.3 可扩展性

- **数据规模**: 单节点支持 100GB+ 数据
- **集群规模**: 支持 3-9 个节点（奇数个，Raft 要求）
- **并发连接**: 支持 10k+ 并发客户端连接

---

## 2. 系统架构

### 2.1 高层架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Layer                              │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│   │  HTTP Client │  │ gRPC Client  │  │  CLI Tool    │         │
│   └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer                                 │
│   ┌──────────────────────────────────────────────────────────┐ │
│   │              Axum Router + Tonic gRPC Server              │ │
│   │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │ │
│   │  │  HTTP Routes │  │  gRPC Routes │  │  Middleware    │  │ │
│   │  │  (RESTful)   │  │  (Protobuf)  │  │  (Auth/Rate)   │  │ │
│   │  └──────────────┘  └──────────────┘  └────────────────┘  │ │
│   └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Engine Layer                                │
│   ┌──────────────────────────────────────────────────────────┐ │
│   │                    KV Engine                              │ │
│   │  ┌─────────────────────────────────────────────────────┐ │ │
│   │  │  Request Validation  │  Command Building             │ │ │
│   │  └─────────────────────────────────────────────────────┘ │ │
│   └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Consensus Layer                               │
│   ┌──────────────────────────────────────────────────────────┐ │
│   │                Consensus Trait                            │ │
│   │  ┌──────────────────┐  ┌──────────────────────────────┐  │ │
│   │  │  NoOpConsensus   │  │    RaftConsensus            │  │ │
│   │  │  (Single Node)   │  │    (Cluster Mode)           │  │ │
│   │  └──────────────────┘  └──────────────────────────────┘  │ │
│   └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Storage Layer                                 │
│   ┌──────────────────────────────────────────────────────────┐ │
│   │              StorageEngine Trait                          │ │
│   │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │ │
│   │  │ InMemoryStore│  │  DiskStore   │  │  SledStore    │  │ │
│   │  │  (HashMap)   │  │  (LSM-Tree)  │  │  (Future)     │  │ │
│   │  └──────────────┘  └──────────────┘  └────────────────┘  │ │
│   └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Network Layer (Cluster Only)                  │
│   ┌──────────────────────────────────────────────────────────┐ │
│   │              Raft Network Transport                       │ │
│   │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │ │
│   │  │  gRPC Proto  │  │  Heartbeat   │  │  Log Replicate │  │ │
│   │  └──────────────┘  └──────────────┘  └────────────────┘  │ │
│   └──────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 核心组件概览

| 组件 | 职责 | 关键技术 |
|------|------|----------|
| **API Layer** | 接收客户端请求，路由到对应处理器 | Axum, Tonic, Tower |
| **Engine Layer** | 业务逻辑处理，命令构建与验证 | - |
| **Consensus Layer** | 分布式一致性保证 | async-raft, Raft protocol |
| **Storage Layer** | 数据持久化与查询 | Trait abstraction, RocksDB/Sled |
| **Network Layer** | 节点间通信（仅集群模式） | gRPC, async-raft network |

### 2.3 部署模式对比

#### 2.3.1 单节点模式

```
┌─────────────────────────────┐
│      Single Node            │
│  ┌───────────────────────┐  │
│  │   API Layer           │  │
│  ├───────────────────────┤  │
│  │   Engine              │  │
│  ├───────────────────────┤  │
│  │   NoOpConsensus       │  │
│  ├───────────────────────┤  │
│  │   StorageEngine       │  │
│  │   (InMemory/Disk)     │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

**特点**:
- 无共识开销，直接写入存储
- 适合开发、测试、小规模生产
- 持久化依赖 StorageEngine 实现

#### 2.3.2 集群模式

```
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│   Node 1     │   │   Node 2     │   │   Node 3     │
│  (Leader)    │◄─►│  (Follower)  │◄─►│  (Follower)  │
│              │   │              │   │              │
│ ┌──────────┐ │   │ ┌──────────┐ │   │ ┌──────────┐ │
│ │   API    │ │   │ │   API    │ │   │ │   API    │ │
│ ├──────────┤ │   │ ├──────────┤ │   │ ├──────────┤ │
│ │ Engine   │ │   │ │ Engine   │ │   │ │ Engine   │ │
│ ├──────────┤ │   │ ├──────────┤ │   │ ├──────────┤ │
│ │  Raft    │ │   │ │  Raft    │ │   │ │  Raft    │ │
│ ├──────────┤ │   │ ├──────────┤ │   │ ├──────────┤ │
│ │ Storage  │ │   │ │ Storage  │ │   │ │ Storage  │ │
│ └──────────┘ │   │ └──────────┘ │   │ └──────────┘ │
└──────────────┘   └──────────────┘   └──────────────┘
        │                  │                  │
        └──────────────────┴──────────────────┘
                     Raft Consensus
```

**特点**:
- Raft 共识保证强一致性
- Leader 处理所有写请求，Follower 可处理读请求（未来优化）
- 容忍 (N-1)/2 个节点故障（3 节点容忍 1 个故障）

### 2.4 技术选型依据

| 技术领域 | 选择 | 替代方案 | 选择理由 |
|----------|------|----------|----------|
| **异步运行时** | Tokio | async-std | Rust 生态系统标准，性能优秀 |
| **HTTP 框架** | Axum | actix-web, warp | Tokio 生态，类型安全，Tower 集成 |
| **gRPC 框架** | Tonic | grpc-rs | Tokio 原生，prost 支持，社区活跃 |
| **Raft 实现** | async-raft | raft-rs | Tokio 原生异步，避免 `spawn_blocking` |
| **序列化** | bincode | serde_json, prost | 性能优先，二进制紧凑 |
| **错误处理** | thiserror + anyhow | failure | 社区标准，thiserror 定义错误，anyhow 处理错误 |

---

## 3. 模块设计

### 3.1 模块结构

```
minikv/
├── src/
│   ├── api/              # API 层
│   │   ├── mod.rs        # 模块导出
│   │   ├── http.rs       # HTTP 路由和处理器
│   │   ├── grpc.rs       # gRPC 服务定义和实现
│   │   └── middleware.rs # 中间件（认证、限流等）
│   │
│   ├── engine/           # 核心引擎
│   │   ├── mod.rs        # KV Engine 主逻辑
│   │   └── command.rs    # 命令定义（Put/Get/Delete）
│   │
│   ├── consensus/        # 共识层
│   │   ├── mod.rs        # Consensus trait 定义
│   │   ├── noop.rs       # NoOpConsensus (单节点)
│   │   └── raft.rs       # RaftConsensus (集群)
│   │
│   ├── storage/          # 存储层
│   │   ├── mod.rs        # StorageEngine trait 定义
│   │   ├── memory.rs     # InMemoryStorage 实现
│   │   └── disk.rs       # DiskStorage 实现
│   │
│   ├── network/          # 网络层（仅集群模式）
│   │   ├── mod.rs        # Raft 网络传输
│   │   └── proto/        # Protobuf 定义
│   │
│   ├── config/           # 配置管理
│   │   ├── mod.rs        # 配置结构和加载
│   │   └── validator.rs  # 配置验证
│   │
│   ├── common/           # 公共类型
│   │   ├── mod.rs
│   │   ├── types.rs      # Key, Value, Result 等类型别名
│   │   └── constants.rs  # 常量定义
│   │
│   ├── errors/           # 错误处理
│   │   ├── mod.rs        # 统一错误类型
│   │   └── ext.rs        # 错误扩展（转换为 HTTP/gRPC 错误）
│   │
│   ├── lib.rs            # 库入口
│   └── main.rs           # 可执行文件入口
│
├── proto/                # Protobuf 定义文件
│   ├── kv.proto          # KV 服务定义
│   └── raft.proto        # Raft 消息定义
│
├── docs/                 # 文档
│   ├── architecture.md   # 架构设计（本文档）
│   └── api.md            # API 文档
│
├── tests/                # 集成测试
│   ├── integration_test.rs
│   └── cluster_test.rs
│
├── benches/              # 性能基准测试
│   └── benchmark.rs
│
├── Cargo.toml            # 项目依赖
└── config/               # 配置文件示例
    ├── single.toml       # 单节点配置
    └── cluster.toml      # 集群配置
```

### 3.2 模块职责与依赖关系

#### 3.2.1 API Layer (`src/api/`)

**职责**:
- 接收 HTTP 和 gRPC 请求
- 请求验证和参数解析
- 调用 Engine 层处理业务逻辑
- 响应格式化和错误处理

**依赖**:
- `engine` - 调用 KV Engine
- `common` - 使用共享类型
- `errors` - 错误转换
- 外部: `axum`, `tonic`, `tower`

**关键接口**:
```rust
// HTTP 路由示例
pub fn create_http_router(engine: Arc<Engine>) -> Router {
    Router::new()
        .route("/kv/:key", get(get_key).put(put_key).delete(delete_key))
        .route("/kv/range", get(scan_range))
        .route("/health", get(health_check))
        .layer(/* middleware */)
        .with_state(engine)
}
```

#### 3.2.2 Engine Layer (`src/engine/`)

**职责**:
- 封装核心业务逻辑
- 构建 Command 对象
- 协调 Consensus 和 Storage 层
- 处理单节点和集群模式的差异

**依赖**:
- `consensus` - 提交命令到共识层
- `storage` - 直接访问存储（单节点模式优化）
- `common` - Command 类型
- `errors` - 业务错误

**关键接口**:
```rust
pub struct Engine<C: Consensus, S: StorageEngine> {
    consensus: C,
    storage: S,
}

impl<C: Consensus, S: StorageEngine> Engine<C, S> {
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // 单节点模式: 直接读取 storage
        // 集群模式: 通过 consensus.read() 保证线性一致性
        self.consensus.read(key).await
    }
    
    pub async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let cmd = Command::Put { key, value };
        self.consensus.propose(cmd).await?;
        Ok(())
    }
}
```

#### 3.2.3 Consensus Layer (`src/consensus/`)

**职责**:
- 定义 Consensus trait
- 实现单节点模式的 NoOpConsensus
- 实现集群模式的 RaftConsensus
- 封装 Raft 状态机

**依赖**:
- `storage` - 应用命令到存储引擎
- `common` - Command 类型
- `errors` - 共识错误
- 外部: `async_raft`

**关键接口**:
```rust
#[async_trait]
pub trait Consensus: Send + Sync + Clone {
    /// 提交写入命令（通过共识协议）
    async fn propose(&self, cmd: Command) -> Result<Vec<u8>>;
    
    /// 强一致性读取（通过 Raft）
    async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// 本地优化读取（不通过 Raft，用于单节点或 leaseholder 优化）
    async fn read_local(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
```

#### 3.2.4 Storage Layer (`src/storage/`)

**职责**:
- 定义 StorageEngine trait
- 实现 InMemoryStorage（测试和开发）
- 实现 DiskStorage（生产环境）
- 管理数据持久化和 WAL

**依赖**:
- `common` - Key/Value 类型
- `errors` - 存储错误
- 外部: `rocksdb` (可选), `sled` (可选)

**关键接口**:
```rust
#[async_trait]
pub trait StorageEngine: Send + Sync + Clone {
    /// 获取单个键值
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// 设置键值（覆盖）
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    
    /// 删除键值
    async fn delete(&self, key: &[u8]) -> Result<()>;
    
    /// 范围扫描
    async fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    
    /// 批量操作（可选）
    async fn batch(&self, ops: Vec<Op>) -> Result<()> {
        // 默认实现：顺序执行
        for op in ops {
            match op {
                Op::Put(k, v) => self.put(k, v).await?,
                Op::Delete(k) => self.delete(&k).await?,
            }
        }
        Ok(())
    }
}
```

#### 3.2.5 Network Layer (`src/network/`)

**职责**:
- 实现 Raft 网络传输接口
- 节点间消息序列化和反序列化
- 处理心跳、日志复制、快照传输

**依赖**:
- `consensus` - Raft 消息处理
- 外部: `tonic`, `prost`

**关键接口**:
```rust
// Raft 网络传输实现
pub struct RaftNetwork {
    nodes: Arc<RwLock<HashMap<u64, NodeClient>>>,
}

#[async_trait]
impl RaftNetworkFactory for RaftNetwork {
    async fn send_message(&self, node_id: u64, msg: RaftMessage) -> Result<()> {
        // 通过 gRPC 发送 Raft 消息到目标节点
    }
}
```

---

## 4. 核心接口设计

### 4.1 StorageEngine Trait

```rust
use async_trait::async_trait;
use crate::errors::Result;

/// 存储引擎操作类型
#[derive(Debug, Clone)]
pub enum Op {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

/// 存储引擎抽象接口
#[async_trait]
pub trait StorageEngine: Send + Sync + Clone + 'static {
    /// 获取键值
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// 设置键值
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    
    /// 删除键值
    async fn delete(&self, key: &[u8]) -> Result<()>;
    
    /// 检查键是否存在
    async fn exists(&self, key: &[u8]) -> Result<bool> {
        Ok(self.get(key).await?.is_some())
    }
    
    /// 范围扫描
    async fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    
    /// 批量操作
    async fn batch(&self, ops: Vec<Op>) -> Result<()> {
        for op in ops {
            match op {
                Op::Put(k, v) => self.put(k, v).await?,
                Op::Delete(k) => self.delete(&k).await?,
            }
        }
        Ok(())
    }
    
    /// 创建快照（用于 Raft）
    async fn snapshot(&self) -> Result<Vec<u8>>;
    
    /// 从快照恢复
    async fn restore(&self, snapshot: Vec<u8>) -> Result<()>;
}
```

### 4.2 Consensus Trait

```rust
use async_trait::async_trait;
use crate::common::Command;
use crate::errors::Result;

/// 共识抽象接口
#[async_trait]
pub trait Consensus: Send + Sync + Clone + 'static {
    /// 提交写入命令
    async fn propose(&self, cmd: Command) -> Result<Vec<u8>>;
    
    /// 强一致性读取
    async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// 本地优化读取
    async fn read_local(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// 获取当前 Leader 信息
    async fn leader(&self) -> Option<u64>;
    
    /// 检查是否为 Leader
    async fn is_leader(&self) -> bool;
}
```

### 4.3 Command Types

```rust
use serde::{Deserialize, Serialize};

/// KV 操作命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// 设置键值
    Put {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    
    /// 删除键值
    Delete {
        key: Vec<u8>,
    },
}

impl Command {
    /// 序列化命令
    pub fn encode(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(Into::into)
    }
    
    /// 反序列化命令
    pub fn decode(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(Into::into)
    }
}
```

### 4.4 Configuration Structures

```rust
use serde::{Deserialize, Serialize};

/// MiniKV 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器配置
    pub server: ServerConfig,
    
    /// 存储配置
    pub storage: StorageConfig,
    
    /// 共识配置
    pub consensus: ConsensusConfig,
    
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP 监听地址
    pub http_addr: String,
    
    /// gRPC 监听地址
    pub grpc_addr: String,
    
    /// 是否启用 HTTP
    pub http_enabled: bool,
    
    /// 是否启用 gRPC
    pub grpc_enabled: bool,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储引擎类型
    pub engine: StorageEngineType,
    
    /// 数据目录（磁盘存储）
    pub data_dir: Option<String>,
    
    /// In-Memory 存储（测试用）
    pub max_memory_mb: Option<usize>,
}

/// 存储引擎类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageEngineType {
    InMemory,
    Disk,
    Sled,
}

/// 共识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// 部署模式
    pub mode: DeployMode,
    
    /// Raft 配置（仅集群模式）
    pub raft: Option<RaftConfig>,
}

/// 部署模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeployMode {
    Single,
    Cluster,
}

/// Raft 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    /// 当前节点 ID
    pub node_id: u64,
    
    /// 集群节点列表
    pub peers: Vec<PeerConfig>,
    
    /// 选举超时（毫秒）
    pub election_timeout_ms: u64,
    
    /// 心跳间隔（毫秒）
    pub heartbeat_interval_ms: u64,
    
    /// 快照阈值
    pub snapshot_threshold: u64,
}

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub node_id: u64,
    pub addr: String,
}
```

### 4.5 示例配置文件

#### 4.5.1 单节点配置 (`config/single.toml`)

```toml
[server]
http_addr = "0.0.0.0:8080"
grpc_addr = "0.0.0.0:9090"
http_enabled = true
grpc_enabled = true

[storage]
engine = "Disk"
data_dir = "/var/lib/minikv/data"

[consensus]
mode = "Single"

[logging]
level = "info"
format = "json"
```

#### 4.5.2 集群配置 (`config/cluster.toml`)

```toml
[server]
http_addr = "0.0.0.0:8080"
grpc_addr = "0.0.0.0:9090"
http_enabled = true
grpc_enabled = true

[storage]
engine = "Disk"
data_dir = "/var/lib/minikv/data"

[consensus]
mode = "Cluster"

[consensus.raft]
node_id = 1
election_timeout_ms = 1000
heartbeat_interval_ms = 100
snapshot_threshold = 10000

[[consensus.raft.peers]]
node_id = 1
addr = "node1:9090"

[[consensus.raft.peers]]
node_id = 2
addr = "node2:9090"

[[consensus.raft.peers]]
node_id = 3
addr = "node3:9090"

[logging]
level = "info"
format = "json"
```

---

## 5. 数据流设计

### 5.1 单节点模式数据流

#### 5.1.1 写入流程

```
Client                API                 Engine          NoOpConsensus    Storage
  │                    │                    │                   │             │
  │──PUT /kv/foo──────►│                    │                   │             │
  │                    │──Engine::put()────►│                   │             │
  │                    │                    │──Command::Put────►│             │
  │                    │                    │                   │──propose()─►│
  │                    │                    │                   │             │──put()
  │                    │                    │                   │             │──WAL
  │                    │                    │                   │◄────────────│ OK
  │                    │                    │                   │             │
  │                    │                    │◄──OK──────────────│             │
  │                    │◄──OK───────────────│                   │             │
  │◄──200 OK───────────│                    │                   │             │
```

**步骤说明**:
1. 客户端发送 HTTP PUT 请求
2. API 层调用 `Engine::put()`
3. Engine 构建 `Command::Put` 对象
4. NoOpConsensus 直接调用 Storage 的 `put()` 方法
5. Storage 写入数据（可选：写入 WAL）
6. 返回成功响应

#### 5.1.2 读取流程

```
Client                API                 Engine          NoOpConsensus    Storage
  │                    │                    │                   │             │
  │──GET /kv/foo──────►│                    │                   │             │
  │                    │──Engine::get()────►│                   │             │
  │                    │                    │──read_local()────►│             │
  │                    │                    │                   │──get()─────►│
  │                    │                    │                   │◄────────────│ value
  │                    │                    │◄──value───────────│             │
  │                    │◄──value────────────│                   │             │
  │◄──200 OK + value───│                    │                   │             │
```

**步骤说明**:
1. 客户端发送 HTTP GET 请求
2. API 层调用 `Engine::get()`
3. NoOpConsensus 调用 `read_local()` 直接读取存储
4. Storage 返回键值
5. 返回响应给客户端

### 5.2 集群模式数据流

#### 5.2.1 写入流程（通过 Raft）

```
Client      API       Engine    RaftConsensus   Raft (Leader)   Network   Raft (Follower)  Storage
  │          │          │            │              │             │              │             │
  │─PUT─────►│          │            │              │             │              │             │
  │          │─put()───►│            │              │             │              │             │
  │          │          │─propose()─►│              │             │              │             │
  │          │          │            │─append log──►│             │              │             │
  │          │          │            │              │─replicate──►│              │             │
  │          │          │            │              │             │─append log──►│             │
  │          │          │            │              │             │◄─ack─────────│             │
  │          │          │            │              │◄─────────────│              │             │
  │          │          │            │              │─commit───────┼──────────────┼─apply─────►│
  │          │          │            │              │              │              │             │──put()
  │          │          │            │              │─apply───────►│              │             │──put()
  │          │          │            │◄─OK──────────│              │              │             │
  │          │          │◄─OK───────│              │              │              │             │
  │          │◄─OK─────│            │              │              │              │             │
  │◄─200 OK──│          │            │              │              │              │             │
```

**步骤说明**:
1. 客户端发送写请求到任意节点
2. 如果不是 Leader，转发到 Leader
3. Leader 将命令追加到 Raft 日志
4. Leader 复制日志到所有 Follower
5. 多数节点确认后，Leader 提交日志
6. 各节点将命令应用到 Storage Engine
7. 返回成功响应给客户端

#### 5.2.2 读取流程（强一致性）

```
Client      API       Engine    RaftConsensus   Raft (Leader)   Storage
  │          │          │            │              │             │
  │─GET─────►│          │            │              │             │
  │          │─get()───►│            │              │             │
  │          │          │─read()────►│              │             │
  │          │          │            │─ReadIndex───►│             │
  │          │          │            │              │─heartbeat──►│ Follower
  │          │          │            │              │◄─ack────────│
  │          │          │            │◄─commit_idx──│             │
  │          │          │            │─apply───────►│             │──get()
  │          │          │            │◄─value───────│             │
  │          │          │◄─value────│              │             │
  │          │◄─value──│            │              │             │
  │◄─200 OK──│          │            │              │             │
```

**步骤说明**:
1. 客户端发送读请求
2. Engine 调用 `Consensus::read()`（强一致性）
3. RaftConsensus 使用 ReadIndex 机制
4. Leader 确认自己是合法 Leader（心跳确认）
5. Leader 读取本地 Storage Engine
6. 返回结果给客户端

#### 5.2.3 读取流程（本地优化，可选）

```
Client      API       Engine    RaftConsensus   Storage
  │          │          │            │             │
  │─GET─────►│          │            │             │
  │          │─get()───►│            │             │
  │          │          │─read_local()─►│          │
  │          │          │            │─get()─────►│
  │          │          │            │◄─value─────│
  │          │          │◄─value────│             │
  │          │◄─value──│            │             │
  │◄─200 OK──│          │            │             │
```

**说明**:
- 用于单节点模式或 Follower Read 优化
- 不经过 Raft 协议，可能读取到过期数据
- 仅在对一致性要求不高的场景使用

### 5.3 故障恢复流程

#### 5.3.1 Leader 故障

```
Time    Node1 (Leader)    Node2 (Follower)    Node3 (Follower)
  │            │                  │                   │
  │────heartbeat timeout──────────────────────────►│
  │            │                  │                   │
  │            │    (Leader crash)│                   │
  │            X                  │                   │
  │                               │──election timeout─│
  │                               │──RequestVote─────►│
  │                               │◄──Vote────────────│
  │                               │                   │
  │                               │  (Become Leader)  │
  │                               │──AppendEntries───►│
  │                               │                   │
```

**步骤说明**:
1. Leader 崩溃，停止发送心跳
2. Follower 选举超时，发起选举
3. 获得多数票后成为新 Leader
4. 新 Leader 开始处理请求

#### 5.3.2 Follower 故障

```
Time    Node1 (Leader)    Node2 (Follower)    Node3 (Follower)
  │            │                  │                   │
  │            │──AppendEntries──►│                   │
  │            │                  X (crash)           │
  │            │──AppendEntries──────────────────────►│
  │            │◄──Success───────────────────────────│
  │            │                  │                   │
  │            │   (continue serving)                 │
  │            │                  │                   │
  │            │──AppendEntries──►│ (recover)         │
  │            │◄──Success────────│                   │
```

**步骤说明**:
1. Follower 崩溃
2. Leader 继续服务，只需多数节点确认
3. Follower 恢复后，Leader 同步缺失的日志

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

## 7. 部署与运维

### 7.1 单节点部署

#### 7.1.1 部署架构

```
┌─────────────────────────────┐
│      Load Balancer          │
│      (Optional)             │
└──────────┬──────────────────┘
           │
           ▼
┌─────────────────────────────┐
│     MiniKV Single Node      │
│  ┌────────────────────────┐ │
│  │  HTTP:8080  gRPC:9090 │ │
│  └────────────────────────┘ │
│  ┌────────────────────────┐ │
│  │   Disk Storage Engine  │ │
│  └────────────────────────┘ │
│          Data: /var/lib/minikv
└─────────────────────────────┘
```

#### 7.1.2 启动命令

```bash
# 启动单节点实例
minikv --config config/single.toml

# 或使用环境变量
MINIKV_CONFIG=config/single.toml minikv
```

#### 7.1.3 Docker 部署

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/minikv /usr/local/bin/
COPY config/single.toml /etc/minikv/config.toml
VOLUME /var/lib/minikv
EXPOSE 8080 9090
CMD ["minikv", "--config", "/etc/minikv/config.toml"]
```

```bash
# 构建镜像
docker build -t minikv:single .

# 运行容器
docker run -d \
  --name minikv-single \
  -p 8080:8080 \
  -p 9090:9090 \
  -v /data/minikv:/var/lib/minikv \
  minikv:single
```

### 7.2 集群部署

#### 7.2.1 部署架构

```
┌─────────────────────────────────────────────────────┐
│                  Load Balancer                       │
│              (Round-robin or Leaseholder)           │
└──────────┬──────────────────────────────────────────┘
           │
    ┌──────┴──────┬──────────────┐
    │             │              │
    ▼             ▼              ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Node 1  │  │ Node 2  │  │ Node 3  │
│ Leader  │◄─►│Follower │◄─►│Follower │
│ :8080   │  │ :8080   │  │ :8080   │
│ :9090   │  │ :9090   │  │ :9090   │
└─────────┘  └─────────┘  └─────────┘
    │             │              │
    └─────────────┴──────────────┘
            Raft Consensus
```

#### 7.2.2 配置管理

每个节点使用不同的配置文件：

**Node 1 (`config/node1.toml`)**:
```toml
[consensus.raft]
node_id = 1

[[consensus.raft.peers]]
node_id = 1
addr = "node1:9090"

[[consensus.raft.peers]]
node_id = 2
addr = "node2:9090"

[[consensus.raft.peers]]
node_id = 3
addr = "node3:9090"
```

**Node 2 和 Node 3** 类似，只需修改 `node_id` 和 `data_dir`。

#### 7.2.3 Docker Compose 部署

```yaml
version: '3.8'

services:
  minikv-node1:
    image: minikv:cluster
    command: minikv --config /etc/minikv/node1.toml
    ports:
      - "8081:8080"
      - "9091:9090"
    volumes:
      - ./config:/etc/minikv
      - /data/minikv/node1:/var/lib/minikv
    networks:
      - minikv-net

  minikv-node2:
    image: minikv:cluster
    command: minikv --config /etc/minikv/node2.toml
    ports:
      - "8082:8080"
      - "9092:9090"
    volumes:
      - ./config:/etc/minikv
      - /data/minikv/node2:/var/lib/minikv
    networks:
      - minikv-net

  minikv-node3:
    image: minikv:cluster
    command: minikv --config /etc/minikv/node3.toml
    ports:
      - "8083:8080"
      - "9093:9090"
    volumes:
      - ./config:/etc/minikv
      - /data/minikv/node3:/var/lib/minikv
    networks:
      - minikv-net

networks:
  minikv-net:
    driver: bridge
```

```bash
# 启动集群
docker-compose up -d

# 查看集群状态
docker-compose ps
```

### 7.3 配置管理

#### 7.3.1 配置文件结构

```
config/
├── single.toml          # 单节点配置
├── cluster.toml         # 集群配置模板
├── node1.toml           # 集群节点 1 配置
├── node2.toml           # 集群节点 2 配置
└── node3.toml           # 集群节点 3 配置
```

#### 7.3.2 环境变量覆盖

```bash
# 覆盖配置文件路径
export MINIKV_CONFIG=/path/to/config.toml

# 覆盖特定配置项
export MINIKV_SERVER_HTTP_ADDR=0.0.0.0:8888
export MINIKV_STORAGE_ENGINE=InMemory
```

### 7.4 监控指标

#### 7.4.1 Prometheus 指标

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "minikv_http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();
    
    static ref REQUEST_LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new("minikv_request_latency_seconds", "Request latency")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
    ).unwrap();
}
```

#### 7.4.2 Grafana Dashboard

示例面板:
- **性能面板**: QPS, 延迟分布, 错误率
- **Raft 面板**: Leader 选举, 日志复制速率, 快照大小
- **存储面板**: 存储大小, 读写 IOPS, 缓存命中率
- **系统面板**: CPU, 内存, 磁盘使用率

---

## 8. 实施路线图

### 8.1 Phase 1: MVP (Week 1-2)

**目标**: 实现最小可用产品，支持单节点模式

#### 8.1.1 功能清单

- [x] 基础项目结构搭建
- [ ] StorageEngine trait 定义
- [ ] InMemoryStorage 实现
- [ ] NoOpConsensus 实现
- [ ] HTTP API (GET/PUT/DELETE)
- [ ] 基本的错误处理
- [ ] 单元测试

#### 8.1.2 技术要点

- 使用 `tokio` 异步运行时
- 使用 `axum` 构建 HTTP 服务
- 使用 `thiserror` 定义错误类型

#### 8.1.3 验收标准

- 能够通过 HTTP API 进行基本的 KV 操作
- 单元测试覆盖率达到 80%+
- 代码通过 clippy 检查

### 8.2 Phase 2: 集群支持 (Week 3-4)

**目标**: 引入 Raft 共识，支持集群模式

#### 8.2.1 功能清单

- [ ] RaftConsensus 实现
- [ ] 集成 `async-raft` 库
- [ ] Raft 网络传输层
- [ ] gRPC API 实现
- [ ] Leader 选举和故障恢复
- [ ] 集群配置管理

#### 8.2.2 技术要点

- 使用 `async-raft` 实现 Raft 协议
- 使用 `tonic` 构建 gRPC 服务
- 实现节点间的心跳和日志复制

#### 8.2.3 验收标准

- 3 节点集群能够正常工作
- 能够容忍 1 个节点故障
- Leader 选举时间 < 10s
- 集成测试通过

### 8.3 Phase 3: 持久化存储 (Week 5-6)

**目标**: 实现磁盘存储引擎，支持数据持久化

#### 8.3.1 功能清单

- [ ] DiskStorage 实现
- [ ] WAL (Write-Ahead Log) 实现
- [ ] SSTable (Sorted String Table) 实现
- [ ] MemTable 到 SSTable 的 Compaction
- [ ] 压缩和快照
- [ ] 性能优化

#### 8.3.2 技术要点

- 实现简单的 LSM-Tree 结构
- 使用 `tokio::fs` 进行异步文件 I/O
- 使用 `crossbeam` 实现并发数据结构

#### 8.3.3 验收标准

- 单节点支持 100GB+ 数据
- 写入吞吐量 > 30k ops/s
- 读取延迟 P99 < 2ms
- 故障恢复后数据完整性验证

### 8.4 Phase 4: 高级特性 (Week 7+)

**目标**: 增强功能和性能优化

#### 8.4.1 功能清单

- [ ] MVCC (多版本并发控制)
- [ ] 事务支持（可选）
- [ ] 动态成员变更
- [ ] 快照传输优化
- [ ] Follower Read 优化
- [ ] 监控和可观测性
- [ ] 性能基准测试

#### 8.4.2 技术要点

- 实现快照隔离级别
- 优化 Raft 日志复制性能
- 集成 Prometheus/Grafana 监控

#### 8.4.3 验收标准

- 支持快照读
- 集群吞吐量 > 20k ops/s
- 完整的监控面板
- 性能报告和优化文档

---

## 9. 附录

### 9.1 参考资料

#### 9.1.1 分布式系统理论

- [Raft Paper](https://raft.github.io/raft.pdf) - In Search of an Understandable Consensus Algorithm
- [Designing Data-Intensive Applications](https://dataintensive.net/) by Martin Kleppmann
- [Distributed Systems: Principles and Paradigms](https://www.distributed-systems.net/) by Tanenbaum & Van Steen

#### 9.1.2 开源项目参考

- [etcd](https://github.com/etcd-io/etcd) - Distributed reliable key-value store
- [TiKV](https://github.com/tikv/tikv) - Distributed transactional key-value database
- [CockroachDB](https://github.com/cockroachdb/cockroach) - The open source, cloud-native SQL database

#### 9.1.3 Rust 生态

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Axum Documentation](https://docs.rs/axum)
- [Tonic Documentation](https://docs.rs/tonic)
- [async-raft Documentation](https://docs.rs/async-raft)

### 9.2 术语表

| 术语 | 英文 | 定义 |
|------|------|------|
| **共识** | Consensus | 分布式系统中多个节点就某个值达成一致的过程 |
| **Raft** | Raft | 一种易于理解的分布式共识算法 |
| **Leader** | Leader | Raft 中负责处理所有客户端请求的节点 |
| **Follower** | Follower | Raft 中被动接收 Leader 日志复制的节点 |
| **Term** | Term | Raft 中的逻辑时间单位，每次选举都会增加 |
| **WAL** | Write-Ahead Log | 预写日志，用于故障恢复 |
| **LSM-Tree** | Log-Structured Merge-Tree | 一种优化写入性能的数据结构 |
| **MVCC** | Multi-Version Concurrency Control | 多版本并发控制，用于事务隔离 |
| **线性一致性** | Linearizability | 最强的一致性保证，所有操作看起来像在单一时间点执行 |

### 9.3 FAQ

#### Q1: 为什么选择 Raft 而不是 Paxos？

**A**: Raft 相比 Paxos 更易于理解和实现，同时提供了相同的正确性保证。Raft 的模块化设计（Leader 选举、日志复制、安全性）使得实现和调试更加容易。

#### Q2: 单节点模式和集群模式如何切换？

**A**: 通过配置文件中的 `consensus.mode` 字段控制。单节点模式使用 `NoOpConsensus`，集群模式使用 `RaftConsensus`。代码层面通过 trait 抽象统一处理，无需修改业务逻辑。

#### Q3: 为什么使用 `async-raft` 而不是 `raft-rs`？

**A**: `async-raft` 是基于 Tokio 的原生异步实现，与我们的异步运行时完美契合。`raft-rs` 是同步实现，需要使用 `spawn_blocking` 包装，可能影响性能。

#### Q4: MVP 阶段为什么只实现 In-Memory 存储？

**A**: In-Memory 实现简单，便于快速验证架构设计和核心逻辑。在 Phase 3 再实现磁盘存储，可以避免过早优化，专注于分布式一致性保证。

#### Q5: 如何保证故障恢复后的数据一致性？

**A**: 通过 WAL 和 Raft 日志的双重保证。单节点模式使用 StorageEngine 内部的 WAL；集群模式使用 Raft 日志复制。故障恢复时，先重放 WAL/Raft 日志，再提供服务。

---

**文档版本历史**:

| 版本 | 日期 | 修改内容 |
|------|------|----------|
| v1.0 | 2026-03-07 | 初始版本 |

---

**文档维护者**: MiniKV Team  
**最后更新**: 2026-03-07