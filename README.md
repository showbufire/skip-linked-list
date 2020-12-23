# skip-linked-list
a skiplist-backed linked list that support fast random writes, written in Rust.

`SkipLinkedList` is a skiplist-backed linked-list that supports fast random access.
The (amortized) time complexity is `O(log n)` for both reads and writes, regardless of the position.
It is more efficient than `Vec` and `Linkedlist` for large list that requires lots of random access.

## Examples
```
let mut list = skip_linked_list::SkipLinkedList::new();

list.push_front(1);
list.push_back(2);
list.insert(1, 3);
list.insert(1, 4);
list.insert(1, 5);
// current list is: [1, 5, 4, 3, 2]

assert_eq!(list.get(1), Some(&5));
assert_eq!(list.get(3), Some(&3));
assert_eq!(list.remove(2), 4);
```
