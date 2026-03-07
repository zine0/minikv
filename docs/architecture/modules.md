# 模块设计


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


---

