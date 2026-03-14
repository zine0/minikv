use std::collections::HashMap;

use crate::errors::Result;
use crate::storage::engine::StorageEngine;
use async_trait::async_trait;

pub struct MemoryEngine {
    store: HashMap<Vec<u8>, Vec<u8>>,
}

impl MemoryEngine {
    /// 创建一个新的内存存储引擎实例
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
}

impl Default for MemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageEngine for MemoryEngine {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let value = self.store.get(key);
        match value {
            Some(val) => Ok(Some(val.to_owned())),
            None => Ok(None),
        }
    }

    async fn put(&mut self, key: Vec<u8>, data: Vec<u8>) -> Result<()> {
        self.store.insert(key, data);
        Ok(())
    }

    async fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.store.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_creates_empty_engine() {
        let engine = MemoryEngine::new();
        let result = engine.get(b"any_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_default_creates_empty_engine() {
        let engine = MemoryEngine::default();
        let result = engine.get(b"any_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_put_and_get() {
        let mut engine = MemoryEngine::new();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        engine.put(key.clone(), value.clone()).await.unwrap();

        let result = engine.get(&key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_delete() {
        let mut engine = MemoryEngine::new();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        engine.put(key.clone(), value).await.unwrap();
        engine.delete(&key).await.unwrap();

        let result = engine.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let engine = MemoryEngine::new();
        let result = engine.get(b"nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_key() {
        let mut engine = MemoryEngine::new();
        let result = engine.delete(b"nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_put_overwrites_existing_key() {
        let mut engine = MemoryEngine::new();
        let key = b"test_key".to_vec();

        engine.put(key.clone(), b"value1".to_vec()).await.unwrap();
        engine.put(key.clone(), b"value2".to_vec()).await.unwrap();

        let result = engine.get(&key).await.unwrap();
        assert_eq!(result, Some(b"value2".to_vec()));
    }

    #[tokio::test]
    async fn test_empty_key() {
        let mut engine = MemoryEngine::new();
        let key: Vec<u8> = Vec::new();
        let value = b"empty_key_value".to_vec();

        engine.put(key.clone(), value.clone()).await.unwrap();
        let result = engine.get(&key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_empty_value() {
        let mut engine = MemoryEngine::new();
        let key = b"empty_value_key".to_vec();
        let value: Vec<u8> = Vec::new();

        engine.put(key.clone(), value.clone()).await.unwrap();
        let result = engine.get(&key).await.unwrap();
        assert_eq!(result, Some(value));
    }

    #[tokio::test]
    async fn test_multiple_keys() {
        let mut engine = MemoryEngine::new();

        engine.put(b"key1".to_vec(), b"value1".to_vec()).await.unwrap();
        engine.put(b"key2".to_vec(), b"value2".to_vec()).await.unwrap();
        engine.put(b"key3".to_vec(), b"value3".to_vec()).await.unwrap();

        assert_eq!(engine.get(b"key1").await.unwrap(), Some(b"value1".to_vec()));
        assert_eq!(engine.get(b"key2").await.unwrap(), Some(b"value2".to_vec()));
        assert_eq!(engine.get(b"key3").await.unwrap(), Some(b"value3".to_vec()));
    }

    #[tokio::test]
    async fn test_binary_keys_and_values() {
        let mut engine = MemoryEngine::new();
        let key = vec![0x00, 0xFF, 0xAB, 0xCD];
        let value = vec![0x01, 0x02, 0x03, 0x04];

        engine.put(key.clone(), value.clone()).await.unwrap();
        let result = engine.get(&key).await.unwrap();
        assert_eq!(result, Some(value));
    }
}
