//https://medium.com/@olcay.d.cabbas/implementing-a-simple-and-efficient-queue-in-rust-using-e33dc9cb9123
//read this article for the queue implementation using raw pointers and box

struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

pub struct Queue<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
    len: usize,
}
impl<T> Queue<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: std::ptr::null_mut(),
            len: 0,
        }
    }
    pub fn enqueue(&mut self, data: T) {
        let mut new_node = Box::new(Node { data, next: None });
        let raw_node: *mut Node<T> = &mut *new_node;
        if self.tail.is_null() {
            self.head = Some(new_node);
        } else {
            unsafe {
                (*self.tail).next = Some(new_node);
            }
        }
        self.tail = raw_node;
        self.len += 1;
    }
    pub fn dequeue(&mut self) -> Option<T> {
        let old_head = self.head.take()?; // self.head=self.head.next ....but wont work cuz mutatble borrow of self.head 
        self.head = old_head.next;
        if self.head.is_none() {
            self.tail = std::ptr::null_mut();
        }
        self.len -= 1;
        Some(old_head.data)
    }
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.data)
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn iterate_over(&self) -> Vec<&T> {
        let mut items = Vec::new();
        let mut current = self.head.as_ref();
        while let Some(node) = current {
            items.push(&node.data);
            current = node.next.as_ref();
        }
        items
    }
}
