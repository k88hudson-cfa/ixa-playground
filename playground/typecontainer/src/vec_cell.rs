use std::{
    alloc::{self, Layout},
    cell::UnsafeCell,
    ptr::{self, NonNull},
};

pub struct VecCell<T: Copy> {
    vec: UnsafeCell<RawVec<T>>,
}

impl<T: Copy> VecCell<T> {
    pub fn new() -> Self {
        VecCell {
            vec: UnsafeCell::new(RawVec::new()),
        }
    }

    pub fn push(&self, val: T) {
        unsafe { (*self.vec.get()).push(val) }
    }

    pub fn get(&self, index: usize) -> T {
        unsafe { (*self.vec.get()).get(index) }
    }

    pub fn set(&self, index: usize, val: T) {
        unsafe { (*self.vec.get()).set(index, val) }
    }
}

struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

impl<T: Copy> RawVec<T> {
    fn new() -> Self {
        // !0 is usize::MAX. This branch should be stripped at compile time.
        let cap = if size_of::<T>() == 0 { !0 } else { 0 };

        // `NonNull::dangling()` doubles as "unallocated" and "zero-sized allocation"
        RawVec {
            ptr: NonNull::dangling(),
            cap,
            len: 0,
        }
    }

    fn grow(&mut self) {
        // since we set the capacity to usize::MAX when T has size 0,
        // getting to here necessarily means the Vec is overfull.
        assert!(size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // This can't overflow because we ensure self.cap <= isize::MAX.
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    fn push(&mut self, val: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), val);
        }

        // Can't fail, we'll OOM first.
        self.len += 1;
    }

    fn get(&self, index: usize) -> T {
        assert!(index < self.len);
        if size_of::<T>() == 0 {
            unsafe { std::mem::zeroed() }
        } else {
            unsafe { *self.ptr.as_ptr().add(index) }
        }
    }

    fn set(&mut self, index: usize, val: T) {
        assert!(index < self.len);
        unsafe {
            ptr::write(self.ptr.as_ptr().add(index), val);
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = size_of::<T>();

        if self.cap != 0 && elem_size != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push_get_set() {
        let vec_cell = VecCell::new();
        vec_cell.push(1);
        assert_eq!(vec_cell.get(0), 1);
        vec_cell.push(2);
        assert_eq!(vec_cell.get(0), 1);
        assert_eq!(vec_cell.get(1), 2);
        vec_cell.set(0, 3);
        assert_eq!(vec_cell.get(0), 3);
        assert_eq!(vec_cell.get(1), 2);
    }

    #[test]
    fn test_zst() {
        let vec_cell = VecCell::new();
        vec_cell.push(());
        assert_eq!(vec_cell.get(0), ());
        vec_cell.push(());
        assert_eq!(vec_cell.get(1), ());
    }
}
