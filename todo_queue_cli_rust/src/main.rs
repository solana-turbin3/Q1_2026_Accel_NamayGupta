use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use todo_queue_cli_rust::{Todo, TodoQueue};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = args[1].as_str();

    let mut queue: TodoQueue<Todo> = TodoQueue::new();
    if let Err(e) = queue.load() {
        eprintln!("Failed to load existing todos: {e}");
    }

    match command {
        "add" => {
            if args.len() < 3 {
                eprintln!("Please provide a description for the todo.");
                return;
            }

            let description = args[2..].join(" ");
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let todo = Todo {
                id: now,
                description,
                created_at: now,
            };

            queue.enqueue(todo);

            if let Err(e) = queue.save() {
                eprintln!("Failed to save todo queue: {e}");
            }
        }
        "list" => {
            if queue.is_empty() {
                println!("No pending todos.");
                return;
            }

            for (index, todo) in queue.iter().iter().enumerate() {
                println!("{index}: {:?}", todo);
            }
        }
        "done" => match queue.dequeue() {
            Some(todo) => {
                println!("Completed: {:?}", todo);
                if let Err(e) = queue.save() {
                    eprintln!("Failed to save todo queue: {e}");
                }
            }
            None => {
                println!("No todos to complete.");
            }
        },
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  todo add \"description\"   - Add a new todo");
    eprintln!("  todo list                 - List all todos");
    eprintln!("  todo done                 - Complete the next todo");
}
