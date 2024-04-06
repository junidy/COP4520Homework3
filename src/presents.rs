use std::time::Instant;
use std::{ptr, thread};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;

struct Node {
    tag: u32,
    next: AtomicPtr<Node>,
}

impl Node {
    fn new(tag: u32) -> Node {
        Node {
            tag,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

struct ConcurrentLinkedList {
    head: AtomicPtr<Node>,
}

impl ConcurrentLinkedList {
    fn new() -> Self {
        ConcurrentLinkedList {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn add_present(&self, tag: u32) {
        let new_node = Box::new(Node::new(tag));
        let new_node_ptr = Box::into_raw(new_node);

        loop {
            let mut prev_ptr = &self.head;
            let mut curr_ptr = prev_ptr.load(Ordering::Acquire);

            // traverse the list to find the correct insertion point
            while !curr_ptr.is_null() {
                let curr = unsafe { &*curr_ptr };

                if curr.tag >= tag {
                    break;
                }

                prev_ptr = &curr.next;
                curr_ptr = curr.next.load(Ordering::Acquire);
            }

            // get the next field of our new node prepped in advance
            unsafe { (*new_node_ptr).next.store(curr_ptr, Ordering::Relaxed); }

            // attempt to insert the new node
            match prev_ptr.compare_exchange_weak(
                curr_ptr,
                new_node_ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break, // success
                Err(_) => continue, // optimism was not rewarded: try again
            }
        }
    }

    fn contains(&self, tag: u32) -> bool {
        let mut curr_ptr = self.head.load(Ordering::Acquire);

        while !curr_ptr.is_null() {
            let curr = unsafe { &*curr_ptr };

            if curr.tag == tag { return true; }
            else if curr.tag > tag { return false; }

            curr_ptr = curr.next.load(Ordering::Acquire);
        }
        false
    }

    fn remove_present(&self, tag: u32) -> bool {
        loop {
            let mut prev_ptr = &self.head;
            let mut curr_ptr = prev_ptr.load(Ordering::Acquire);

            // look for the ndoe to remove
            while !curr_ptr.is_null() {
                let curr = unsafe { &*curr_ptr };
                if curr.tag == tag { break; }
                prev_ptr = &curr.next;
                curr_ptr = curr.next.load(Ordering::Acquire);
            }

            // exit if couldn't find node
            if curr_ptr.is_null() { return false; }

            // try to remove the node
            let curr = unsafe { &*curr_ptr };
            let next_ptr = curr.next.load(Ordering::Acquire);

            match prev_ptr.compare_exchange_weak(curr_ptr, next_ptr, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => { return true; } // removed successfully
                Err(_) => continue, // try again
            }
        }
    }
}

pub fn sort_presents(num_presents: usize, num_servants: usize) {
    let list = Arc::new(ConcurrentLinkedList::new());
    let presents_added = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();

    println!("Starting sort...");
    let now = Instant::now();

    for _ in 0..num_servants {
        let list_clone = Arc::clone(&list);
        let presents_added_clone = Arc::clone(&presents_added);
        threads.push(thread::spawn(move || {
            while presents_added_clone.load(Ordering::SeqCst) < num_presents {
                let next_present = presents_added_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if next_present <= num_presents {
                    list_clone.add_present(next_present as u32);
                    list_clone.remove_present(next_present as u32);
                }
            }
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }

    println!("{} presents were added and removed by {} servants in {:.2?} seconds", num_presents, num_servants, now.elapsed().as_secs_f32());
}   