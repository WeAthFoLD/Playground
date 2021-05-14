use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

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

type ThreadPoolEntry = Box<dyn Fn() + Send + 'static>;

struct ThreadPool {
    queue: Arc<RingBuffer<ThreadPoolEntry>>
}

impl ThreadPool {

    fn queue_task<F>(&self, task: F) where F: Fn() + Send + 'static {
        match self.queue.push(Box::new(task)) {
            Result::Ok(_) => (), 
            Result::Err(_) => panic!("Thread pending queue is full")
        }
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
    test_queue();
}
