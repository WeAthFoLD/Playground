use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering, AtomicBool};
use std::cell::UnsafeCell;
use std::thread;

/// 一个勉强能用的RingBuffer
struct RingBuffer<T> where T: Sized + Send {
    /// 使用 UnsafeCell 让我们可以在 &self 里对 Vec 进行操作。
    /// 这样才能在正常的使用里避免多线程加锁（不然就需要Arc<RwLock<RingBuffer>>>，破坏了无锁队列的初衷。。）
    arr: Vec<UnsafeCell<Option<T>>>,
    head: AtomicUsize,
    tail: AtomicUsize
}

impl<T> RingBuffer<T> where T: Sized + Send {

    fn new(size: usize) -> Self {
        assert!(size > 1);
        let mut v = Vec::new();
        for _ in 0..size {
            v.push(None.into());
        }
        Self {
            arr: v,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0) 
        }
    }

    fn push(&self, val: T) -> Result<(), T> {
        let size = self.size();
        // tail = (tail + 1) % (size)
        // if tail + 1 == head (with modulo): overflow

        // CAS
        let mut last_tail = self.tail.load(Ordering::Relaxed);
        loop {
            let nlast = (last_tail + 1) % size;
            // 在cas里比较队列是否满，若满直接返回
            let head = self.head.load(Ordering::Relaxed);
            // println!("  push head={}, tail={}", head, last_tail);
            if (last_tail + 1 + size - head) % size == 0 {
                return Result::Err(val);
            }
            match self.tail.compare_exchange(last_tail, nlast, 
                Ordering::Release, Ordering::Relaxed) {
                Result::Ok(_) => {
                    break;
                },
                Result::Err(x) => last_tail = x
            }
        }

        let slot = last_tail;
        unsafe { // SAFETY: 多个线程不会访问同一个cell
            *self.arr[slot].get() = Some(val)
        }
        Result::Ok(())
    }

    fn pop(&self) -> Result<T, ()> {
        let size = self.size();

        let mut last_head = self.head.load(Ordering::Relaxed);
        loop {
            let nlast = (last_head + 1) % size;
            // 在cas里比较队列是否空，如果是，直接返回
            let tail = self.tail.load(Ordering::Relaxed);
            if last_head == tail {
                return Result::Err(());
            }

            match self.head.compare_exchange(last_head, nlast,
                Ordering::Acquire, Ordering::Relaxed) {
                Result::Ok(_) => {
                    // last_head = x;
                    break;
                },
                Result::Err(x) => last_head = x,
            }
        }

        let mut elem: Option<T> = None;
        unsafe { // SAFETY: 多个线程不会访问同一个cell
            let ptr = self.arr[last_head].get();
            std::ptr::swap((&mut elem) as *mut Option<T>, ptr);
        }

        match elem {
            Some(x) => Result::Ok(x),
            None => Result::Err(())
        }
    }

    fn size(&self) -> usize {
        self.arr.len()
    }

}

/// (实际不会被sync，绕过编译检查用)
unsafe impl<T: Sized + Send> Sync for RingBuffer<T> {}

type ThreadPoolEntry = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    queue: Arc<RingBuffer<ThreadPoolEntry>>,
    destroyed_flag: Arc<AtomicBool>,
    child_threads: Vec<thread::JoinHandle<()>>
}

impl ThreadPool {

    fn new(thread_count: usize) -> Self {
        let destroyed_flag = Arc::new(AtomicBool::new(false));
        let queue = Arc::new(RingBuffer::<ThreadPoolEntry>::new(16));
        let mut child_threads = vec![];
        for i in 0..thread_count {
            let sub_destroyed_flag = destroyed_flag.clone();
            let sub_queue = queue.clone();
            let join_handle = thread::spawn(move || {
                loop {
                    match sub_queue.pop() {
                        Result::Ok(task) => {
                            task();
                        },
                        _ => {
                            if sub_destroyed_flag.load(Ordering::Relaxed) {
                                break
                            } else {
                                thread::yield_now()
                            }
                        }
                    }
                }

                println!("Thread #{} destroyed.", i);
            });

            child_threads.push(join_handle);
        }

        Self {
            queue: queue,
            destroyed_flag,
            child_threads
        }
    }

    fn queue_task<F>(&self, task: F) where F: FnOnce() + Send + 'static {
        match self.queue.push(Box::new(task)) {
            Result::Ok(_) => (), 
            Result::Err(_) => panic!("Thread pending queue is full")
        }
    }

    /// join实现方式不是很优雅，因为实现了Drop所以无法将成员变量move out，
    /// 所以只能用swap vec的方式拿到所有thread handle。
    fn join(mut self) {
        self.destroyed_flag.store(true, Ordering::Relaxed);

        let mut v = Vec::new();
        std::mem::swap(&mut v, &mut self.child_threads);
        for handle in v {
            handle.join().expect("Failed to join thread");
        }
    }

}

impl Drop for ThreadPool {

    fn drop(&mut self) {
        self.destroyed_flag.store(true, Ordering::Relaxed);
    }

}

fn test_queue() {
    println!("Test queue: single case");
    {
        let mut q: RingBuffer<u32> = RingBuffer::new(8);
        for i in 0..7 {
            println!("Push {}", i);
            assert_eq!(q.push(i), Result::Ok(()));
        }

        println!("Push 7 (out of range)");
        assert_eq!(q.push(7), Result::Err(7));

        for i in 0..4 {
            match q.pop() {
                Result::Ok(x) => println!("Pop returns {}", x),
                _ => panic!()
            }
        }

        println!("Push 5");
        assert_eq!(q.push(5), Result::Ok( () ));
        for i in 0..5 {
            match q.pop() {
                Result::Ok(x) => println!("Pop returns {}", x),
                _ => println!("Pop returns None")
            }
        }

        for i in 0..7 {
            println!("Push {}", i + 8);
            assert_eq!(q.push(i), Result::Ok(()));
        }

        for i in 0..7 {
            match q.pop() {
                Result::Ok(x) => println!("Pop returns {}", x),
                _ => println!("Pop returns None")
            }
        }
    }
}

fn main() {
    use std::time::Duration;
    println!("Testing ringbuffer...");
    test_queue();

    println!();
    println!("Testing thread pool...");
    let thread_pool = ThreadPool::new(3);
    for i in 0..4 {
        thread_pool.queue_task(move || {
            for j in 0..5 {
                thread::sleep(Duration::from_millis(1000));
                println!("Task #{} say {}", i, j);
            }
        });
    }

    thread_pool.join();
}
