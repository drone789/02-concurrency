// metrics data structure
// 基本功能: inc/dec/snopshot

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct Metrics {
    data: Arc<RwLock<HashMap<String, i64>>>,
    // data: Arc<Mutex<HashMap<String, i64>>>,
    // Arc 用于多线程环境下共享 Metrics 结构体的所有权，确保多个线程可以同时访问 Metrics 实例
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        // lock() 方法返回一个 Result<MutexGuard<T>, PoisonError<..>>，
        // 其中 MutexGuard 是一个智能指针，用于在获取到锁之后访问 Mutex 中的数据。
        // 如果 lock() 方法执行成功，会返回一个 MutexGuard 对象，表示成功获取到了锁。
        // 如果 lock() 方法执行失败，会返回一个 PoisonError 对象，表示获取锁失败。
        // 通过使用 map_err(|e| anyhow!(e.to_string()))，
        // 在出现错误时将错误信息转换为一个 anyhow 的 Result 类型，这样可以更加方便地处理错误，同时保留了错误的具体信息，有助于调试和排查问题。
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?;

        // ?
        // self.data.lock() 成功获取到了锁，返回一个 Ok(MutexGuard<T>)
        // self.data.lock() 获取锁失败，返回一个 Err，? 会立即将这个 Err 返回给调用者，中断当前函数的执行，并将这个 Err 作为整个函数的返回值

        // data实现了DerefMut trait,
        // data.entry() = HashMap<String,i64>.entry()
        let counter = data.entry(key.into()).or_insert(0);
        *counter += 1;
        Ok(())
    }

    // pub fn dec(&mut self, key: &str) {
    //     let counter = self.data.entry(key.to_string()).or_insert(0);
    //     *counter -= 1;
    // }

    pub fn snopshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .read()
            .map_err(|e| anyhow!(e.to_string()))?
            .clone())
    }
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = self.data.read().map_err(|_e| fmt::Error {})?;
        for (key, value) in data.iter() {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}
