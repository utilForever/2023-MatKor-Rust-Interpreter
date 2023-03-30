#[cfg(test)]
use std::rc::Rc;

pub struct CircularBuffer<T> {
    data: Vec<Option<T>>,
    read_index: usize,
    write_index: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyBuffer,
    FullBuffer,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: (0..capacity).map(|_| None).collect(),
            read_index: 0,
            write_index: 0,
        }
    }

    pub fn write(&mut self, element: T) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error::FullBuffer);
        }

        self.write_without_check(element);
        Ok(())
    }

    pub fn read(&mut self) -> Result<T, Error> {
        self.data[self.read_index]
            .take()
            .ok_or(Error::EmptyBuffer)
            .map(|value| {
                self.read_index = self.increase_index(self.read_index);
                value
            })
    }

    pub fn clear(&mut self) {
        self.data = (0..self.data.len()).map(|_| None).collect();
        self.read_index = 0;
        self.write_index = 0;
    }

    pub fn overwrite(&mut self, element: T) {
        let is_overwriting = self.is_full();
        self.write_without_check(element);
        if is_overwriting {
            self.read_index = self.increase_index(self.read_index);
        }
    }

    fn is_full(&self) -> bool {
        self.data[self.write_index].is_some()
    }

    fn increase_index(&self, index: usize) -> usize {
        (index + 1) % self.data.len()
    }

    fn write_without_check(&mut self, element: T) {
        self.data[self.write_index] = Some(element);
        self.write_index = self.increase_index(self.write_index);
    }
}

#[test]
fn error_on_read_empty_buffer() {
    let mut buffer = CircularBuffer::<char>::new(1);
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn can_read_item_just_written() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    assert_eq!(Ok('1'), buffer.read());
}

#[test]
fn each_item_may_only_be_read_once() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    assert_eq!(Ok('1'), buffer.read());
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn items_are_read_in_the_order_they_are_written() {
    let mut buffer = CircularBuffer::new(2);
    assert!(buffer.write('1').is_ok());
    assert!(buffer.write('2').is_ok());
    assert_eq!(Ok('1'), buffer.read());
    assert_eq!(Ok('2'), buffer.read());
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn full_buffer_cant_be_written_to() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    assert_eq!(Err(Error::FullBuffer), buffer.write('2'));
}

#[test]
fn read_frees_up_capacity_for_another_write() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    assert_eq!(Ok('1'), buffer.read());
    assert!(buffer.write('2').is_ok());
    assert_eq!(Ok('2'), buffer.read());
}

#[test]
fn read_position_is_maintained_even_across_multiple_writes() {
    let mut buffer = CircularBuffer::new(3);
    assert!(buffer.write('1').is_ok());
    assert!(buffer.write('2').is_ok());
    assert_eq!(Ok('1'), buffer.read());
    assert!(buffer.write('3').is_ok());
    assert_eq!(Ok('2'), buffer.read());
    assert_eq!(Ok('3'), buffer.read());
}

#[test]
fn items_cleared_out_of_buffer_cant_be_read() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    buffer.clear();
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn clear_frees_up_capacity_for_another_write() {
    let mut buffer = CircularBuffer::new(1);
    assert!(buffer.write('1').is_ok());
    buffer.clear();
    assert!(buffer.write('2').is_ok());
    assert_eq!(Ok('2'), buffer.read());
}

#[test]
fn clear_does_nothing_on_empty_buffer() {
    let mut buffer = CircularBuffer::new(1);
    buffer.clear();
    assert!(buffer.write('1').is_ok());
    assert_eq!(Ok('1'), buffer.read());
}

#[test]
fn clear_actually_frees_up_its_elements() {
    let mut buffer = CircularBuffer::new(1);
    let element = Rc::new(());
    assert!(buffer.write(Rc::clone(&element)).is_ok());
    assert_eq!(Rc::strong_count(&element), 2);
    buffer.clear();
    assert_eq!(Rc::strong_count(&element), 1);
}

#[test]
fn overwrite_acts_like_write_on_non_full_buffer() {
    let mut buffer = CircularBuffer::new(2);
    assert!(buffer.write('1').is_ok());
    buffer.overwrite('2');
    assert_eq!(Ok('1'), buffer.read());
    assert_eq!(Ok('2'), buffer.read());
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn overwrite_replaces_the_oldest_item_on_full_buffer() {
    let mut buffer = CircularBuffer::new(2);
    assert!(buffer.write('1').is_ok());
    assert!(buffer.write('2').is_ok());
    buffer.overwrite('A');
    assert_eq!(Ok('2'), buffer.read());
    assert_eq!(Ok('A'), buffer.read());
}

#[test]
fn overwrite_replaces_the_oldest_item_remaining_in_buffer_following_a_read() {
    let mut buffer = CircularBuffer::new(3);
    assert!(buffer.write('1').is_ok());
    assert!(buffer.write('2').is_ok());
    assert!(buffer.write('3').is_ok());
    assert_eq!(Ok('1'), buffer.read());
    assert!(buffer.write('4').is_ok());
    buffer.overwrite('5');
    assert_eq!(Ok('3'), buffer.read());
    assert_eq!(Ok('4'), buffer.read());
    assert_eq!(Ok('5'), buffer.read());
}

#[test]
fn integer_buffer() {
    let mut buffer = CircularBuffer::new(2);
    assert!(buffer.write(1).is_ok());
    assert!(buffer.write(2).is_ok());
    assert_eq!(Ok(1), buffer.read());
    assert!(buffer.write(-1).is_ok());
    assert_eq!(Ok(2), buffer.read());
    assert_eq!(Ok(-1), buffer.read());
    assert_eq!(Err(Error::EmptyBuffer), buffer.read());
}

#[test]
fn string_buffer() {
    let mut buffer = CircularBuffer::new(2);
    buffer.write("".to_string()).unwrap();
    buffer.write("Testing".to_string()).unwrap();
    assert_eq!(0, buffer.read().unwrap().len());
    assert_eq!(Ok("Testing".to_string()), buffer.read());
}
