extern crate rand;

use rand::{thread_rng, Rng};
use std::ptr::NonNull;
use std::fmt::Display;

pub struct SkipLinkedList<T> {
    size: usize,
    entry: Link<T>,
}

type Link<T> = Box<Node<T>>;
type WeakLink<T> = NonNull<Node<T>>;

enum Node<T> {
    Sentinel { right: Option<Link<T>>, down: Option<Link<T>>, delta: usize },
    Index { right: Option<Link<T>>, down: WeakLink<T>, delta: usize },
    Content { right: Option<Link<T>>, elem: T },
}

impl<T> SkipLinkedList<T> {
    pub fn new() -> Self {
        Self {
            size: 0,
            entry: Box::new(Node::Sentinel { right: None, down: None, delta: 1}),
        }
    }

    pub fn insert(&mut self, i: usize, elem: T) -> bool {
        if i > self.size {
            return false;
        }

        let i = i + 1; // relative to sentinel
        let top_level_inserted = Node::insert(&mut self.entry, i, elem);
        self.size += 1;
        match (top_level_inserted, thread_rng().gen_bool(0.5)) {
            (Some(raw_node), true) => {
                let new_index = Node::Index { right: None, down: raw_node, delta: self.size - i + 1 };
                let mut entry = Box::new(Node::Sentinel { right: Some(Box::new(new_index)), down: None, delta: i });
                std::mem::swap(&mut self.entry, &mut entry);
                match self.entry.as_mut() {
                    Node::Sentinel { down, .. } => *down = Some(entry),
                    _ => (),
                }
            },
            _ => (),
        }

        return true;
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.size {
            return None;
        }
        Node::get(&self.entry, i + 1)
    }

    pub fn remove(&mut self, _i: usize) -> T {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn push_front(&mut self, elem: T) {
        self.insert(0, elem);
    }

    pub fn push_back(&mut self, elem: T) {
        self.insert(self.size, elem);
    }

}

const WIDTH: usize = 4;

impl<T> SkipLinkedList<T> where T: Display {
    pub fn visualize(&self) {
        let mut option_node = Some(&self.entry);
        while let Some(node) = option_node.take() {
            Self::visualize_level(Some(node));
            match node.as_ref() {
                Node::Sentinel { down, .. } => option_node = down.as_ref(),
                _ => break,
            }
        }
    }

    fn visualize_level(option_node: Option<&Box<Node<T>>>) {
        let mut option_node = option_node;
        let mut last_delta = 0;
        while let Some(node) = option_node.take() {
            match node.as_ref() {
                Node::Sentinel { right, delta, .. } => {
                    print!("{delta:>width$}", delta=format!("+{}", delta), width=WIDTH);
                    last_delta = *delta;
                    option_node = right.as_ref();
                },
                Node::Index { right, delta, .. } => {
                    print!("{delta:>width$}", delta=format!("+{}", delta), width=(last_delta*WIDTH));
                    last_delta = *delta;
                    option_node = right.as_ref();
                },
                Node::Content { right, elem, .. } => {
                    print!("{elem:>width$}", elem=elem, width=WIDTH);
                    option_node = right.as_ref();
                },
            }
        }
        println!();
    }
}

impl<T> Node<T> {
    fn right_mut(&mut self) -> &mut Option<Link<T>> {
        match self {
            Node::Sentinel { right, .. } => right,
            Node::Content { right, .. }  => right,
            Node::Index { right, .. } => right,
        }
    }

    fn right(&self) -> Option<&Link<T>> {
        match self {
            Node::Sentinel { right, .. } => right.as_ref(),
            Node::Content { right, .. }  => right.as_ref(),
            Node::Index { right, .. } => right.as_ref(),
        }
    }

    fn insert(start_node: &mut Node<T>, start_i: usize, elem: T) -> Option<WeakLink<T>> {
        let mut node = start_node;
        let mut i = start_i;

        while node.delta() < i {
            i -= node.delta();
            node = node.right_mut().as_mut().unwrap();
        }
        node.insert_at(i, elem)
    }

    fn get(start_node: &Node<T>, start_i: usize) -> Option<&T> {
        let mut node = start_node;
        let mut i = start_i;

        while node.delta() <= i {
            i -= node.delta();
            node = node.right().unwrap();
        }
        node.get_at(i)
    }

    fn get_at(&self, i: usize) -> Option<&T> {
        match self {
            Node::Sentinel { down: Some(node), .. } => Node::get(node, i),
            Node::Index { down: raw_node, .. } => Node::get(unsafe { raw_node.as_ref() }, i),
            Node::Content { elem, .. } if i == 0 => Some(&elem),
            _ => None,
        }
    }

    fn insert_content_after(&mut self, elem: T) -> Option<WeakLink<T>> {
        let right = self.right_mut();
        let mut new_node = Box::new(Node::Content { elem, right: right.take() });
        let raw_new_node: *mut _ = &mut *new_node;
        *right = Some(new_node);
        NonNull::new(raw_new_node)
    }

    fn insert_index_after(&mut self, i: usize, next_level_inserted: WeakLink<T>) -> Option<WeakLink<T>> {
        let delta = self.delta();
        let right = self.right_mut();
        let mut new_node = Box::new(Node::Index {
            right: right.take(),
            down: next_level_inserted,
            delta: delta - i,
        });
        let raw_new_node: *mut _ = &mut *new_node;
        *right = Some(new_node);
        *self.delta_mut().unwrap() = i;
        NonNull::new(raw_new_node)
    }

    fn insert_at(&mut self, i: usize, elem: T) -> Option<WeakLink<T>> {
        match self {
            Node::Content { .. } => self.insert_content_after(elem),
            Node::Sentinel { down: None, .. } => self.insert_content_after(elem),
            Node::Sentinel { down: Some(node), delta, .. } => {
                *delta += 1;
                match (Node::insert(node, i, elem), thread_rng().gen_bool(0.5)) {
                    (Some(next_level_inserted), true) => self.insert_index_after(i, next_level_inserted),
                    _ => None,
                }
            },
            Node::Index { down: raw_node, delta, .. } => {
                *delta += 1;
                match (Node::insert(unsafe { raw_node.as_mut() }, i, elem), thread_rng().gen_bool(0.5)) {
                    (Some(next_level_inserted), true) => self.insert_index_after(i, next_level_inserted),
                    _ => None,
                }
            },
        }
    }

    fn delta(&self) -> usize {
        match self {
            Node::Sentinel { delta, .. } => *delta,
            Node::Content { .. } => 1,
            Node::Index { delta, .. } => *delta,
        }
    }

    fn delta_mut(&mut self) -> Option<&mut usize> {
        match self {
            Node::Sentinel { delta, .. } => Some(delta),
            Node::Content { .. } => None,
            Node::Index { delta, .. } => Some(delta),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        let mut list = SkipLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_front(30);
        list.push_front(20);
        list.push_front(10);
        list.insert(3, 100);
        assert_eq!(list.len(), 7);
        let expected = vec![10, 20, 30, 100, 1, 2, 3];
        for (i, elem) in expected.iter().enumerate() {
            assert_eq!(list.get(i), Some(elem));
        }
        assert_eq!(list.get(10), None);
    }
}