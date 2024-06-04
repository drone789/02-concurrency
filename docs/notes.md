# 并发编程

## 1 使用 Arc::new(Mutex::new())

```
Arc 里面的数据只能读
Mutex.lock() -> MutexGuard !send 不能在线程之前传递
退出时,mutex 会自动 unlock()
```

## 2 RwLock

- Mutex
  - lock()
- RwLock
  - read()
  - write()

> 基本是从 Mutex 平滑替换到 RwLock

## 3 DashMap 支持并发的 HashMap

## 4 AtomicXXX

- fetch // 读
- load // 写
