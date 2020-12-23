extern crate rand;

use rand::{thread_rng, Rng};
use std::ptr::NonNull;
use std::fmt::Display;

/// # SkipLinkedList
///
/// `SkipLinkedList` is a skiplist-backed linked-list that supports fast random access.
/// The (amortized) time complexity is `O(log n)` for both reads and writes, regardless of the position.
/// It is more efficient than `Vec` and `Linkedlist` for large list that requires lots of random access.
///
/// # Examples
/// ```
/// let mut list = skip_linked_list::SkipLinkedList::new();
///
/// list.push_front(1);
/// list.push_back(2);
/// list.insert(1, 3);
/// list.insert(1, 4);
/// list.insert(1, 5);
/// // current list is: [1, 5, 4, 3, 2]
///
/// assert_eq!(list.get(1), Some(&5));
/// assert_eq!(list.get(3), Some(&3));
/// assert_eq!(list.remove(2), 4);
/// ```
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

    /// Creates a new list.
    pub fn new() -> Self {
        Self {
            size: 0,
            entry: Box::new(Node::Sentinel { right: None, down: None, delta: 1}),
        }
    }

    /// Inserts an element at position index within the list, shifting all elements after it to the right.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = skip_linked_list::SkipLinkedList::new();
    /// list.insert(0, 10);
    /// list.insert(1, 30);
    /// list.insert(1, 20);
    /// assert_eq!(list.into_iter().collect::<Vec<i32>>(), vec![10, 20, 30]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `i > len`.
    pub fn insert(&mut self, i: usize, elem: T) {
        if i > self.size {
            panic!("insert position {} should be <= len (is {})", i, self.size);
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
    }

    /// Gets the element at position index within the list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = skip_linked_list::SkipLinkedList::new();
    /// list.insert(0, 10);
    /// assert_eq!(list.get(0), Some(&10));
    /// assert_eq!(list.get(1), None);
    /// ```
    pub fn get(&self, i: usize) -> Option<&T> {
        if i >= self.size {
            return None;
        }
        Node::get(&self.entry, i + 1)
    }

    /// Removes an element at position index within the list, shifting all elements after it to the left.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = skip_linked_list::SkipLinkedList::new();
    /// list.insert(0, 10);
    /// list.insert(1, 20);
    /// assert_eq!(list.remove(0), 10);
    /// assert_eq!(list.remove(0), 20);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `i >= len`.

    pub fn remove(&mut self, i: usize) -> T {
        if i >= self.size {
            panic!("remove position {} should be < len (is {})", i, self.size);
        }
        self.size -= 1;
        Node::remove(&mut self.entry, i)
    }

    /// Returns the length of the list.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Inserts an element at the start of the list.
    pub fn push_front(&mut self, elem: T) {
        self.insert(0, elem);
    }

    /// Inserts an element at the end of the list.
    pub fn push_back(&mut self, elem: T) {
        self.insert(self.size, elem);
    }

    /// Removes an element at the start of the list.
    /// # Panics
    ///
    /// Panics if list is empty.
    pub fn pop_front(&mut self) -> T {
        if self.size > 0 {
            self.remove(0)
        } else {
            panic!("can't pop an empty list")
        }
    }

    /// Removes an element at the end of the list.
    /// # Panics
    ///
    /// Panics if list is empty.
    pub fn pop_back(&mut self) -> T {
        if self.size > 0 {
            self.remove(self.size - 1)
        } else {
            panic!("can't pop an empty list")
        }
    }

    /// Returns an iterator over the list.
    pub fn iter(&self) -> Iter<T> {
        let mut node = self.entry.as_ref();
        while let Node::Sentinel{ down: Some(next_node), .. } = node {
            node = next_node;
        }
        Iter(node.right())
    }

    /// Returns an mut iterator over the list.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        let mut node = self.entry.as_mut();
        while let Node::Sentinel{ down: Some(next_node), .. } = node {
            node = next_node;
        }
        IterMut(node.right_mut().as_mut())
    }

    /// Consumes the list into an iterator.
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

pub struct IntoIter<T>(SkipLinkedList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() > 0 {
            Some(self.0.pop_front())
        } else {
            None
        }
    }
}

pub struct IterMut<'a, T>(Option<&'a mut Link<T>>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().and_then(|node| {
            if let Node::Content { elem, right } = node.as_mut() {
                self.0 = right.as_mut();
                Some(elem)
            } else {
                None
            }
        })
    }
}

pub struct Iter<'a, T>(Option<&'a Link<T>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take().and_then(|node| {
            if let Node::Content { elem, right } = node.as_ref() {
                self.0 = right.as_ref();
                Some(elem)
            } else {
                None
            }
        })
    }
}

const WIDTH: usize = 4;

impl<T> SkipLinkedList<T> where T: Display {

    /// Prints the internals of the list.
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
            Node::Content { .. } | Node:: Sentinel { down: None, .. } => self.insert_content_after(elem),
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

    fn remove(start_node: &mut Node<T>, i: usize) -> T {
        let mut i = i;
        let mut node = start_node;

        while node.delta() <= i {
            i -= node.delta();
            node = node.right_mut().as_mut().unwrap();
        }
        node.remove_after(i)
    }

    fn remove_after(&mut self, i: usize) -> T {
        match self {
            Node::Sentinel { down: Some(node), delta, .. } => {
                let removed = Node::remove(node, i);
                if *delta == i + 1 {
                    self.remove_right();
                } else {
                    *delta -= 1;
                };
                removed
            },
            Node::Index { down: raw_node, delta, .. } => {
                let removed = Node::remove(unsafe { raw_node.as_mut() }, i);
                if *delta == i + 1 {
                    self.remove_right();
                } else {
                    *delta -= 1;
                }
                removed
            },
            Node::Sentinel { down: None, .. } => self.remove_right().unwrap(),
            Node::Content {.. } => self.remove_right().unwrap(),
        }
    }

    fn remove_right(&mut self) -> Option<T> {
        let right = self.right_mut();
        let mut removed = right.take().unwrap();
        *right = removed.right_mut().take();
        self.delta_mut().map (|delta| *delta += removed.delta() - 1);
        match *removed {
            Node::Content { elem, .. } => Some(elem),
            _ => None,
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

    fn drop_after(sentinel: &mut Node<T>) {
        sentinel.right_mut().take().map(|mut node| {
            while let Some(next_node) = node.right_mut().take() {
                node = next_node;
            }
        });
        if let Node::Sentinel { down: Some(next_sentinel), .. } = sentinel {
            Node::drop_after(next_sentinel);
        }
    }
}

impl<T> Drop for SkipLinkedList<T> {
    fn drop(&mut self) {
        Node::drop_after(&mut self.entry);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_list() -> SkipLinkedList<i32> {
        let mut list = SkipLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_front(30);
        list.push_front(20);
        list.push_front(10);
        list.insert(3, 100);
        list
    }

    #[test]
    fn basics() {
        let mut list = setup_list();
        assert_eq!(list.len(), 7);
        let expected = vec![10, 20, 30, 100, 1, 2, 3];
        for (i, elem) in expected.iter().enumerate() {
            assert_eq!(list.get(i), Some(elem));
        }
        assert_eq!(list.get(10), None);
        assert_eq!(list.remove(0), 10);
        assert_eq!(list.remove(0), 20);
        assert_eq!(list.remove(4), 3);
        assert_eq!(list.remove(2), 1);
    }

    #[test]
    fn small_random() {
        let mut list = SkipLinkedList::new();
        let mut vec = Vec::new();

        let mut size = 0;
        for _ in 0..1000 {
            size += 1;
            let elem: i32 = thread_rng().gen();
            let idx: usize = thread_rng().gen_range(0, size);
            list.insert(idx, elem);
            vec.insert(idx, elem);
        }
        assert_eq!(list.len(), vec.len());
        for i in 0..1000 {
            assert_eq!(list.get(i), vec.get(i));
        }
    }

    #[test]
    fn iter() {
        let list = setup_list();
        let mut iter = list.iter();
        let expected = vec![10, 20, 30, 100, 1, 2, 3];
        for elem in expected.iter() {
            assert_eq!(iter.next(), Some(elem));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = setup_list();
        let mut iter_mut = list.iter_mut();
        while let Some(elem) = iter_mut.next() {
            *elem += 1;
        }
        let expected = vec![11, 21, 31, 101, 2, 3, 4];
        let mut iter = list.iter();
        for elem in expected.iter() {
            assert_eq!(iter.next(), Some(elem));
        }
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn into_iter() {
        let list = setup_list();
        let expected = vec![10, 20, 30, 100, 1, 2, 3];
        let mut into_iter = list.into_iter();

        for elem in expected {
            assert_eq!(into_iter.next(), Some(elem));
        }
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn drop() {
        let size = 50000;
        let mut list = SkipLinkedList::new();
        for _ in 0..size {
            list.push_front(1);
        }
    }

    #[test]
    fn pops() {
        let mut list = SkipLinkedList::new();
        list.push_front(1);
        list.push_front(2);
        assert_eq!(list.pop_front(), 2);
        assert_eq!(list.pop_front(), 1);

        list.push_back(1);
        list.push_back(2);
        assert_eq!(list.pop_back(), 2);
        assert_eq!(list.pop_back(), 1);
    }

    #[test]
    #[should_panic]
    fn panic_pop_front() {
        let mut list: SkipLinkedList<i32> = SkipLinkedList::new();
        list.pop_front();
    }

    #[test]
    #[should_panic]
    fn panic_pop_back() {
        let mut list: SkipLinkedList<i32> = SkipLinkedList::new();
        list.pop_back();
    }

    #[test]
    #[should_panic]
    fn panic_insert() {
        let mut list = SkipLinkedList::new();
        list.insert(1, 3);
    }
}