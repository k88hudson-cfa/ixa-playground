use std::cell::UnsafeCell;

pub struct VecCell<T> {
    vec: UnsafeCell<Vec<T>>,
}

impl<T: Copy> VecCell<T> {
    pub fn new() -> Self {
        VecCell {
            vec: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn push(&self, value: T) {
        let vec = unsafe { &mut *self.vec.get() };
        vec.push(value);
    }

    pub fn get(&self, index: usize) -> T {
        let vec = unsafe { &*self.vec.get() };
        *vec.get(index).unwrap()
    }

    pub fn get_or_extend(&self, index: usize, value: T) -> T {
        let vec = unsafe { &mut *self.vec.get() };
        if index >= vec.len() {
            vec.resize(index + 1, value);
        }
        *vec.get(index).unwrap()
    }

    pub fn set(&self, index: usize, value: T) {
        let vec = unsafe { &mut *self.vec.get() };
        vec[index] = value;
    }

    pub fn set_or_extend(&self, index: usize, value: T, default: T) {
        let vec = unsafe { &mut *self.vec.get() };
        if index >= vec.len() {
            vec.resize(index + 1, default);
        }
        let vec = unsafe { &mut *self.vec.get() };
        vec[index] = value;
    }
}
