
实现一个ThreadPool，用于学习多线程执行相关概念以及在rust里的应用。

# Basic Concept

ThreadPool提供以下操作：

* new(thread_count: usize) -> 创建一个ThreadPool
* queue(closure: Fn) where Fn: Send -> 排队一项操作，在有空闲线程时操作被执行
* 不提供内置的synchronization机制

实现：

* 核心是一个基于Atomic的**ringbuffer无锁队列**
* `ThreadPool` 自己是 `Send + !Sync`；可以跨线程共用（用`Arc<Mutex<Thread>>`）