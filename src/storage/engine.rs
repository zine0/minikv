use crate::errors::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StorageEngine {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    async fn put(&mut self, key: Vec<u8>, data: Vec<u8>) -> Result<()>;

    async fn delete(&mut self, key: &[u8]) -> Result<()>;
}
