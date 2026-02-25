use borsh::{BorshDeserialize, BorshSerialize};
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
mod queue;
use crate::queue::Queue;

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Todo {
    pub id: u64,
    pub description: String,
    pub created_at: u64,
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
    pub fn iter(&self) -> Vec<&T> {
        self.items.iterate_over()
    }
}

impl<T> TodoQueue<T>
where
    T: BorshDeserialize + BorshSerialize + Clone,
{
    pub fn save(&self) -> Result<(), Error> {
        let mut file = File::create("todo.bin")?;
        let items_refs = self.items.iterate_over();
        let items: Vec<T> = items_refs.into_iter().cloned().collect();
        let bytes = borsh::to_vec(&items)
            .map_err(|e| Error::new(ErrorKind::Other, format!("serialization error: {e}")))?;
        file.write_all(&bytes)
    }
    pub fn load(&mut self) -> Result<(), Error> {
        let mut file = match File::open("todo.bin") {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // First run: no file yet, treat as empty queue.
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let items: Vec<T> = borsh::from_slice(&bytes)
            .map_err(|e| Error::new(ErrorKind::Other, format!("deserialization error: {e}")))?;

        self.items = Queue::new();
        for item in items {
            self.items.enqueue(item);
        }

        Ok(())
    }
}
