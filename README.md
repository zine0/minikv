# MiniKV

一个基于 Rust 和 Tokio 构建的分布式键值存储系统。

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## 特性

- 🚀 **高性能**: 基于 Tokio 异步运行时，支持高并发
- 🔒 **强一致性**: 通过 Raft 共识算法保证分布式一致性
- 🔧 **灵活部署**: 支持单节点和集群两种部署模式
- 🔌 **可插拔存储**: 通过 trait 抽象支持多种存储后端（In-Memory、自定义磁盘存储）
- 🌐 **多协议 API**: 同时提供 HTTP RESTful 和 gRPC 接口

## 项目状态

🚧 **设计阶段** - 目前正在进行架构设计，尚未开始实现。

## 架构设计

详细的架构设计文档请参阅 [docs/architecture.md](docs/architecture.md)。

### 核心架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer                                 │
│              (HTTP + gRPC)                                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Engine Layer                                │
│              (KV Engine)                                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Consensus Layer                               │
│        (NoOpConsensus / RaftConsensus)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Storage Layer                                 │
│     (InMemoryStorage / DiskStorage)                             │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 环境要求

- Rust 1.75+
- Tokio 1.x
- Protobuf 编译器（用于 gRPC）

### 构建项目

```bash
# 克隆项目
git clone https://github.com/your-org/minikv.git
cd minikv

# 构建项目
cargo build

# 运行测试
cargo test

# 运行单节点模式
cargo run -- --config config/single.toml

# 运行集群模式（需要 3 个节点）
cargo run -- --config config/node1.toml
```

### API 示例

#### HTTP API

```bash
# 设置键值
curl -X PUT http://localhost:8080/kv/foo -d "bar"

# 获取键值
curl http://localhost:8080/kv/foo

# 删除键值
curl -X DELETE http://localhost:8080/kv/foo

# 范围查询
curl "http://localhost:8080/kv/range?start=a&end=z"
```

#### gRPC API

```protobuf
syntax = "proto3";

service KVService {
  rpc Get(GetRequest) returns (GetResponse);
  rpc Put(PutRequest) returns (PutResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
  rpc Scan(ScanRequest) returns (stream KeyValue);
}

message GetRequest {
  bytes key = 1;
}

message GetResponse {
  bytes value = 1;
}

message PutRequest {
  bytes key = 1;
  bytes value = 2;
}

message PutResponse {}

message DeleteRequest {
  bytes key = 1;
}

message DeleteResponse {}

message ScanRequest {
  bytes start = 1;
  bytes end = 2;
  int32 limit = 3;
}

message KeyValue {
  bytes key = 1;
  bytes value = 2;
}
```

## 配置

### 单节点配置

```toml
[server]
http_addr = "0.0.0.0:8080"
grpc_addr = "0.0.0.0:9090"

[storage]
engine = "InMemory"

[consensus]
mode = "Single"

[logging]
level = "info"
```

### 集群配置

```toml
[server]
http_addr = "0.0.0.0:8080"
grpc_addr = "0.0.0.0:9090"

[storage]
engine = "Disk"
data_dir = "/var/lib/minikv/data"

[consensus]
mode = "Cluster"

[consensus.raft]
node_id = 1
election_timeout_ms = 1000
heartbeat_interval_ms = 100

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

## 开发路线图

### Phase 1: MVP (Week 1-2) - 🚧 进行中
- [x] 架构设计
- [ ] 基础项目结构
- [ ] StorageEngine trait
- [ ] InMemoryStorage 实现
- [ ] NoOpConsensus 实现
- [ ] HTTP API (GET/PUT/DELETE)

### Phase 2: 集群支持 (Week 3-4)
- [ ] RaftConsensus 实现
- [ ] gRPC API
- [ ] 集群部署

### Phase 3: 持久化存储 (Week 5-6)
- [ ] DiskStorage 实现
- [ ] WAL 和 SSTable
- [ ] Compaction

### Phase 4: 高级特性 (Week 7+)
- [ ] MVCC
- [ ] 事务支持
- [ ] 性能优化

## 文档

- [架构设计文档](docs/architecture/README.md) - 详细的系统架构设计（已按模块拆分）
  - [概述](docs/architecture/01-overview.md) - 项目目标和设计原则
  - [系统架构](docs/architecture/02-architecture.md) - 高层架构和组件设计
  - [模块设计](docs/architecture/03-modules.md) - 代码结构和职责划分
  - [核心接口](docs/architecture/04-interfaces.md) - Trait 定义和接口设计
  - [数据流设计](docs/architecture/05-data-flow.md) - 读写流程和故障恢复
  - [技术栈](docs/architecture/06-tech-stack.md) - 核心依赖和工具链
  - [部署与运维](docs/architecture/07-deployment.md) - 部署方案和配置管理
  - [实施路线图](docs/architecture/08-roadmap.md) - 开发计划和时间表
  - [附录](docs/architecture/09-appendix.md) - 参考资料、术语表、FAQ
- [API 文档](docs/api.md) - HTTP 和 gRPC API 文档（待创建）

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 致谢

本项目的设计受到了以下项目的启发：

- [etcd](https://github.com/etcd-io/etcd) - 分布式键值存储
- [TiKV](https://github.com/tikv/tikv) - 分布式事务键值数据库
- [CockroachDB](https://github.com/cockroachdb/cockroach) - 分布式 SQL 数据库

## 联系方式

- 问题反馈: [GitHub Issues](https://github.com/your-org/minikv/issues)
- 讨论交流: [GitHub Discussions](https://github.com/your-org/minikv/discussions)