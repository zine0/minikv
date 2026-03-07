# 数据流设计


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


---

