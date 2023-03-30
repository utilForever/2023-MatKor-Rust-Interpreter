use std::{marker::PhantomData, ptr};

mod linked_list;

pub struct Node<T> {
    data: T,
    next: *mut Node<T>,
    prev: *mut Node<T>,
}

pub struct LinkedList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
}

pub struct Cursor<'a, T> {
    list: &'a mut LinkedList<T>,
    curr: *mut Node<T>,
}

pub struct Iter<'a, T> {
    curr: *mut Node<T>,
    _marker: PhantomData<&'a Node<T>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Return a cursor positioned on the front element
    pub fn cursor_front(&mut self) -> Cursor<'_, T> {
        let head_ptr: *mut _ = self.head;
        Cursor {
            list: self,
            curr: head_ptr,
        }
    }

    /// Return a cursor positioned on the back element
    pub fn cursor_back(&mut self) -> Cursor<'_, T> {
        let tail_ptr: *mut _ = self.tail;
        Cursor {
            list: self,
            curr: tail_ptr,
        }
    }

    /// Return an iterator that moves from front to back
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            curr: self.head,
            _marker: PhantomData,
        }
    }
}

// the cursor is expected to act as if it is at the position of an element
// and it also has to work with and be able to insert into an empty list.
impl<T> Cursor<'_, T> {
    /// Take a mutable reference to the current element
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        if self.curr.is_null() {
            return None;
        }

        unsafe { Some(&mut (*self.curr).data) }
    }

    /// Move one position forward (towards the back) and
    /// return a reference to the new position
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&mut T> {
        unsafe {
            if self.curr.is_null() || (*self.curr).next.is_null() {
                return None;
            }

            self.curr = (*self.curr).next;
            Some(&mut (*self.curr).data)
        }
    }

    /// Move one position backward (towards the front) and
    /// return a reference to the new position
    pub fn prev(&mut self) -> Option<&mut T> {
        unsafe {
            if self.curr.is_null() || (*self.curr).prev.is_null() {
                return None;
            }

            self.curr = (*self.curr).prev;
            Some(&mut (*self.curr).data)
        }
    }

    /// Remove and return the element at the current position and move the cursor
    /// to the neighboring element that's closest to the back. This can be
    /// either the next or previous position.
    pub fn take(&mut self) -> Option<T> {
        let curr_ptr = self.curr;

        if curr_ptr.is_null() {
            return None;
        }

        unsafe {
            let next_node_ptr = (*curr_ptr).next;
            let prev_node_ptr = (*curr_ptr).prev;

            if !next_node_ptr.is_null() && prev_node_ptr.is_null() {
                // Head
                (*next_node_ptr).prev = ptr::null_mut();
                self.list.head = next_node_ptr;
                self.curr = next_node_ptr;
            } else if next_node_ptr.is_null() && !prev_node_ptr.is_null() {
                // Tail
                (*prev_node_ptr).next = ptr::null_mut();
                self.list.tail = prev_node_ptr;
                self.curr = prev_node_ptr;
            } else if !next_node_ptr.is_null() && !prev_node_ptr.is_null() {
                // Middle
                (*prev_node_ptr).next = next_node_ptr;
                (*next_node_ptr).prev = prev_node_ptr;
                self.curr = next_node_ptr;
            } else {
                // Only one
                self.curr = ptr::null_mut();
                self.list.head = ptr::null_mut();
                self.list.tail = ptr::null_mut();
            }

            self.list.len -= 1;

            let data = std::ptr::read(&(*curr_ptr).data);
            drop(Box::from_raw(curr_ptr));

            Some(data)
        }
    }

    pub fn insert_after(&mut self, element: T) {
        let new_node = Box::new(Node {
            data: element,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        });

        let new_node_ptr: *mut _ = Box::into_raw(new_node);

        if !self.curr.is_null() {
            unsafe {
                (*new_node_ptr).prev = self.curr;
                let next_node_ptr = (*self.curr).next;

                if !next_node_ptr.is_null() {
                    (*new_node_ptr).next = next_node_ptr;
                    (*next_node_ptr).prev = new_node_ptr;
                } else {
                    self.list.tail = new_node_ptr;
                }

                (*self.curr).next = new_node_ptr;
            }
        } else {
            self.list.head = new_node_ptr;
            self.list.tail = new_node_ptr;
            self.curr = new_node_ptr;
        }

        self.list.len += 1;
    }

    pub fn insert_before(&mut self, element: T) {
        let new_node = Box::new(Node {
            data: element,
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        });

        let new_node_ptr: *mut _ = Box::into_raw(new_node);

        if !self.curr.is_null() {
            unsafe {
                (*new_node_ptr).next = self.curr;
                let prev_node_ptr = (*self.curr).prev;

                if !prev_node_ptr.is_null() {
                    (*new_node_ptr).prev = prev_node_ptr;
                    (*prev_node_ptr).next = new_node_ptr;
                } else {
                    self.list.head = new_node_ptr;
                }

                (*self.curr).prev = new_node_ptr;
            }
        } else {
            self.list.head = new_node_ptr;
            self.list.tail = new_node_ptr;
            self.curr = new_node_ptr;
        }

        self.list.len += 1;
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        unsafe {
            if self.curr.is_null() {
                return None;
            }

            let data = &(*self.curr).data;
            self.curr = (*self.curr).next;
            Some(data)
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut curr_ptr = self.head;

        while !curr_ptr.is_null() {
            unsafe {
                let next_ptr = (*curr_ptr).next;
                drop(Box::from_raw(curr_ptr));
                curr_ptr = next_ptr;
            }
        }
    }
}

#[test]
fn is_generic() {
    struct Foo;
    LinkedList::<Foo>::new();
}

// ———————————————————————————————————————————————————————————
// Tests for Step 1: push / pop at front and back
// ———————————————————————————————————————————————————————————

#[test]
fn basics_empty_list() {
    let list: LinkedList<i32> = LinkedList::new();
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

// push / pop at back ————————————————————————————————————————
#[test]
fn basics_single_element_back() {
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push_back(5);

    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());

    assert_eq!(list.pop_back(), Some(5));

    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn basics_push_pop_at_back() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_back(i);
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
    }
    for i in (0..10).rev() {
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
        assert_eq!(i, list.pop_back().unwrap());
    }
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

// push / pop at front ———————————————————————————————————————
#[test]
fn basics_single_element_front() {
    let mut list: LinkedList<i32> = LinkedList::new();
    list.push_front(5);

    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());

    assert_eq!(list.pop_front(), Some(5));

    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn basics_push_pop_at_front() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_front(i);
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
    }
    for i in (0..10).rev() {
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
        assert_eq!(i, list.pop_front().unwrap());
    }
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

// push / pop at mixed sides —————————————————————————————————
#[test]
fn basics_push_front_pop_back() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_front(i);
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
    }
    for i in 0..10 {
        assert_eq!(list.len(), 10 - i as usize);
        assert!(!list.is_empty());
        assert_eq!(i, list.pop_back().unwrap());
    }
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

#[test]
fn basics_push_back_pop_front() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for i in 0..10 {
        list.push_back(i);
        assert_eq!(list.len(), i as usize + 1);
        assert!(!list.is_empty());
    }
    for i in 0..10 {
        assert_eq!(list.len(), 10 - i as usize);
        assert!(!list.is_empty());
        assert_eq!(i, list.pop_front().unwrap());
    }
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

// ———————————————————————————————————————————————————————————
// Tests for Step 2: iteration
// ———————————————————————————————————————————————————————————

#[test]
fn iter() {
    let mut list: LinkedList<i32> = LinkedList::new();
    for num in 0..10 {
        list.push_back(num);
    }

    for (num, &entered_num) in (0..10).zip(list.iter()) {
        assert_eq!(num, entered_num);
    }
}

// ———————————————————————————————————————————————————————————
// Tests for Step 3: full cursor functionality
// ———————————————————————————————————————————————————————————

#[test]
fn cursor_insert_before_on_empty_list() {
    // insert_after on empty list is already tested via push_back()
    let mut list = LinkedList::new();
    list.cursor_front().insert_before(0);
    assert_eq!(Some(0), list.pop_front());
}

#[test]
fn cursor_insert_after_in_middle() {
    let mut list = (0..10).collect::<LinkedList<_>>();

    {
        let mut cursor = list.cursor_front();
        let didnt_run_into_end = cursor.seek_forward(4);
        assert!(didnt_run_into_end);

        for n in (0..10).rev() {
            cursor.insert_after(n);
        }
    }

    assert_eq!(list.len(), 20);

    let expected = (0..5).chain(0..10).chain(5..10);

    assert!(expected.eq(list.iter().cloned()));
}

#[test]
fn cursor_insert_before_in_middle() {
    let mut list = (0..10).collect::<LinkedList<_>>();

    {
        let mut cursor = list.cursor_back();
        let didnt_run_into_end = cursor.seek_backward(4);
        assert!(didnt_run_into_end);

        for n in 0..10 {
            cursor.insert_before(n);
        }
    }

    assert_eq!(list.len(), 20);

    let expected = (0..5).chain(0..10).chain(5..10);

    assert!(expected.eq(list.iter().cloned()));
}

// "iterates" via next() and checks that it visits the right elements
#[test]
fn cursor_next_and_peek() {
    let mut list = (0..10).collect::<LinkedList<_>>();
    let mut cursor = list.cursor_front();

    assert_eq!(cursor.peek_mut(), Some(&mut 0));

    for n in 1..10 {
        let next = cursor.next().cloned();
        assert_eq!(next, Some(n));
        assert_eq!(next, cursor.peek_mut().cloned());
    }
}

// "iterates" via prev() and checks that it visits the right elements
#[test]
fn cursor_prev_and_peek() {
    let mut list = (0..10).collect::<LinkedList<_>>();
    let mut cursor = list.cursor_back();

    assert_eq!(cursor.peek_mut(), Some(&mut 9));

    for n in (0..9).rev() {
        let prev = cursor.prev().cloned();
        assert_eq!(prev, Some(n));
        assert_eq!(prev, cursor.peek_mut().cloned());
    }
}

// removes all elements starting from the middle
#[test]
fn cursor_take() {
    let mut list = (0..10).collect::<LinkedList<_>>();
    let mut cursor = list.cursor_front();
    cursor.seek_forward(5);

    for expected in (5..10).chain((0..5).rev()) {
        assert_eq!(cursor.take(), Some(expected));
    }
}

// ———————————————————————————————————————————————————————————
// Tests for Step 4: clean-up via `Drop`
// ———————————————————————————————————————————————————————————

// The leak tests that are also for this step are separated into
// their own files so that nothing else interferes with the allocator
// whilst they run

// checks number of drops
// may pass for incorrect programs if double frees happen
// exactly as often as destructor leaks
#[test]
fn drop_no_double_frees() {
    use std::cell::Cell;
    struct DropCounter<'a>(&'a Cell<usize>);

    impl<'a> Drop for DropCounter<'a> {
        fn drop(&mut self) {
            let num = self.0.get();
            self.0.set(num + 1);
        }
    }

    const N: usize = 15;

    let counter = Cell::new(0);
    let list = std::iter::repeat_with(|| DropCounter(&counter))
        .take(N)
        .collect::<LinkedList<_>>();

    assert_eq!(list.len(), N);
    drop(list);
    assert_eq!(counter.get(), N);
}

#[test]
fn drop_large_list() {
    drop((0..2_000_000).collect::<LinkedList<i32>>());
}

// ———————————————————————————————————————————————————————————
// Tests for Step 5 (advanced): covariance and Send/Sync
// ———————————————————————————————————————————————————————————

// These are compile time tests. They won't compile unless your
// code passes.

#[cfg(feature = "advanced")]
#[test]
fn advanced_linked_list_is_send_sync() {
    trait AssertSend: Send {}
    trait AssertSync: Sync {}

    impl<T: Send> AssertSend for LinkedList<T> {}
    impl<T: Sync> AssertSync for LinkedList<T> {}
}

#[cfg(feature = "advanced")]
#[allow(dead_code)]
#[test]
fn advanced_is_covariant() {
    fn a<'a>(x: LinkedList<&'static str>) -> LinkedList<&'a str> {
        x
    }

    fn a_iter<'a>(i: Iter<'static, &'static str>) -> Iter<'a, &'a str> {
        i
    }
}
