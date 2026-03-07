# 附录

[返回主页](./README.md) | [上一章：实施路线图](./08-roadmap.md)

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
---

[返回主页](./README.md) | [上一章：实施路线图](./08-roadmap.md)
