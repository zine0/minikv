use std::io;

use thiserror::Error;
pub type Result<T> = std::result::Result<T, KvError>;

#[derive(Error, Debug)]
pub enum KvError {
    /// 键不存在
    #[error("Key not found: {0}")]
    NotFound(String),

    /// 底层 I/O 错误（如读写文件失败）
    #[error("Storage I/O error: {0}")]
    Io(#[from] io::Error),

    /// 序列化/反序列化错误（例如使用 serde_json）
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 数据格式无效（如存储的数据损坏）
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// 事务冲突（如写冲突或死锁）
    #[error("Transaction conflict")]
    TransactionConflict,

    /// 其他未分类的错误，保留扩展性
    #[error("Other error: {0}")]
    Other(String),
}
