# 核心接口设计

[返回主页](./README.md) | [上一章：模块设计](./03-modules.md) | [下一章：数据流设计](./05-data-flow.md)

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


---

[返回主页](./README.md) | [上一章：模块设计](./03-modules.md) | [下一章：数据流设计](./05-data-flow.md)
