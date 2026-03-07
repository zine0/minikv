# 部署与运维


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


---

