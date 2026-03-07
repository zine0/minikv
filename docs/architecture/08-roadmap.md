# 实施路线图


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


---

