use borsh::{BorshDeserialize, BorshSerialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
mod queue;
use crate::queue::Queue;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Todo {
    id: u64,
    description: String,
    created_at: u64,
}

pub struct TodoQueue<T> {
    items: Queue<T>,
}

impl<T> TodoQueue<T> {
    pub fn new() -> Self {
        Self {
            items: Queue::new(),
        }
    }
    pub fn enqueue(&mut self, item: T) {
        self.items.enqueue(item);
        //println!("Todo added to queue: {:?}", item);
    }
    pub fn dequeue(&mut self) -> Option<T> {
        self.items.dequeue()
    }
    pub fn peek(&self) -> Option<&T> {
        self.items.peek()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> TodoQueue<T>
where
    T: BorshDeserialize + BorshSerialize,
{
    pub fn save(&self) -> Result<(), std::io::Error> {
        let mut file = File::create("todo.bin")?;
        let items = self.items.iterate_over();
        let bytes = borsh::to_vec(&items)?;
        file.write_all(&bytes)
    }
    // pub fn load(&mut self) -> Result<(), std::io::Error> {}
}
