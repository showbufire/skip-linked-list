use std::io::{self, Write};

mod skip_linked_list;
use crate::skip_linked_list::SkipLinkedList;

fn main() {
    let mut list = SkipLinkedList::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        match line.as_bytes()[0] {
            b'i' => {
                let mut parts = line[1..].split_whitespace().map(|s| s.parse::<i32>());
                match (parts.next(), parts.next()) {
                    (Some(Ok(i)), Some(Ok(elem))) => {
                        if list.insert(i as usize, elem) {
                            println!("inserted");
                        } else {
                            println!("fail to insert");
                        }
                    },
                    _ => println!("Unknown command, type h for help"),
                }
            },
            b'f' => {
                match line[1..].trim().parse::<i32>() {
                    Ok(elem) => list.push_front(elem),
                    _ => println!("Unknown command, type h for help"),
                }
            },
            b'b' => {
                match line[1..].trim().parse::<i32>() {
                    Ok(elem) => list.push_back(elem),
                    _ => println!("Unknown command, type h for help"),
                }
            },
            b'l' => println!("{}", list.len()),
            b'p' => list.visualize(),
            b'c' => list = SkipLinkedList::new(),
            b'x' => break,
            b'h' => {
                println!("insert: i idx elem");
                println!("push front: f elem");
                println!("push back: b elem");
                println!("print: p");
                println!("len: l");
                println!("c: clear");
                println!("exit: x");
            },
            _ => println!("Unknown command, type h for help"),
        }
    }
}