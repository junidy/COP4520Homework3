use std::{sync::atomic::AtomicBool, thread};

type Tag = usize;

struct Present {
    tag: Tag,
    lock: AtomicBool,
    next: Box<Option<Present>>
}

impl Present {
    pub fn with_id(id: Tag) -> Present {
        Present {
            tag: id,
            lock: AtomicBool::new(false),
            next: Box::<Option<Present>>::new(None)
        }
    }
}

pub struct PresentChain {
    head: Box<Option<Present>>
}

impl PresentChain {
    pub fn new() -> PresentChain {
        PresentChain { 
            head: Box::new(None)
        }
    }
    // pub fn add(&mut self, id: Tag) {
    //     let curr = 
    //     // iterate through chain
    // }
    pub fn add(&self, tag: Tag) {
        let new_present = Box::new(Present::new(tag));
        let new_present_ptr = Box::into_raw(new_present);

        loop {
            // Try to find the insertion point
            let mut prev_ptr = &self.head;
            let mut curr_ptr = prev_ptr.load(Ordering::Acquire);

            while !curr_ptr.is_null() {
                let curr = unsafe { &*curr_ptr };

                if curr.tag >= tag {
                    // Correct spot found
                    break;
                }
                prev_ptr = &curr.next;
                curr_ptr = curr.next.load(Ordering::Acquire);
            }

            // Set the new present's next pointer
            unsafe { (*new_present_ptr).next.store(curr_ptr, Ordering::Release); }

            // Try to insert the new present
            match prev_ptr.compare_exchange_weak(
                curr_ptr,
                new_present_ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return, // Successfully added
                Err(_) => {
                    // Failed to add; likely lost to another thread. Retry.
                    continue;
                }
            }
        }
    }
    
    pub fn remove(&mut self, id: Tag) {

    }
    pub fn contains(&self, id: Tag) {

    }
}

// pub fn sort_presents(num_presents: usize, num_servants: usize) {
//     let mut servant_handles = Vec::new();
//     let present
//     let chain = Arc::new(PresentChain::new());
//     for i in 0..num_servants {
//         chain_clone = chain.clone();
//         servant_handles.push(thread::spawn(move || {

//         }));
//     }
// }
