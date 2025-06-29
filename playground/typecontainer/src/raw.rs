use crate::Key;
use std::{
    alloc::{self, Layout},
    any::TypeId,
    cell::UnsafeCell,
    hash::{DefaultHasher, Hash, Hasher},
    ptr,
};

pub struct TypeContainer {
    container: UnsafeCell<RawTypeContainer>,
}

impl TypeContainer {
    pub fn new() -> TypeContainer {
        TypeContainer {
            container: UnsafeCell::new(RawTypeContainer::new()),
        }
    }

    pub fn try_insert<K: Key>(&self, value: K::Value) -> Result<(), String> {
        let container = unsafe { &mut *self.container.get() };
        if container.contains::<K>() {
            return Err("Container already contains key".into());
        }
        // Safety: all container entries are heap allocated and this insertion
        //  cannot invalidate any existing pointers
        unsafe { Ok((*self.container.get()).insert::<K>(value)) }
    }

    pub fn get<K: Key>(&self) -> Option<&K::Value> {
        unsafe { (*self.container.get()).get::<K>() }
    }

    pub fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        unsafe { (*self.container.get()).get_mut::<K>() }
    }
}

type AnyPtr = *const ();
type TypeEntry = (TypeId, AnyPtr, Layout);

struct RawTypeContainer {
    ptr: *mut Option<TypeEntry>,
    capacity: usize,
    length: usize,
}

impl RawTypeContainer {
    fn new() -> RawTypeContainer {
        RawTypeContainer {
            ptr: std::ptr::null_mut(),
            capacity: 0,
            length: 0,
        }
    }

    fn grow(&mut self) {
        let (new_capacity, new_layout) = if self.capacity == 0 {
            // Start with minimum size 4
            (4, Layout::array::<Option<TypeEntry>>(4).unwrap())
        } else {
            // This can't overflow because we ensure self.cap <= isize::MAX.
            let new_capacity = 2 * self.capacity;

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<Option<TypeEntry>>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = unsafe { alloc::alloc(new_layout) as *mut Option<TypeEntry> };
        for i in 0..new_capacity {
            unsafe { ptr::write(new_ptr.add(i), None) }
        }

        if self.capacity > 0 {
            // Need to rehash all entries into new array
            for i in 0..self.capacity {
                if let Some((key, value, layout)) = unsafe { ptr::read(self.ptr.add(i)) } {
                    let mut hasher = DefaultHasher::new();
                    key.hash(&mut hasher);
                    let hash = hasher.finish();
                    let mut index = (hash % (new_capacity as u64)) as usize;
                    let mut iter = 0;
                    while iter < new_capacity {
                        let entry = unsafe { &mut *new_ptr.add(index) };
                        if entry.is_none() {
                            let _ = entry.insert((key, value, layout));
                            break;
                        }
                        iter += 1;
                        index = (index + iter) % new_capacity;
                    }
                }
            }

            // Dealloc old array
            let old_layout = Layout::array::<Option<TypeEntry>>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    fn contains<K: Key>(&self) -> bool {
        if self.capacity == 0 {
            return false;
        }

        let type_id = TypeId::of::<K>();
        // Todo: Use faster hash
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = (hash % (self.capacity as u64)) as usize;
        let mut iter = 0;
        while iter < self.capacity {
            let entry = unsafe { *self.ptr.add(index) };
            match entry {
                Some((key, _, _)) => {
                    if key.eq(&type_id) {
                        return true;
                    }
                    // Continue probing
                }
                None => {
                    return false;
                }
            }
            iter += 1;
            index = (index + iter) % self.capacity;
        }
        return false;
    }

    fn get<K: Key>(&self) -> Option<&K::Value> {
        if self.capacity == 0 {
            return None;
        }

        let type_id = TypeId::of::<K>();
        // Todo: Use faster hash
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = (hash % (self.capacity as u64)) as usize;
        let mut iter = 0;
        while iter < self.capacity {
            let entry = unsafe { *self.ptr.add(index) };
            match entry {
                Some((key, ptr, _)) => {
                    if key.eq(&type_id) {
                        let value = unsafe { &*(ptr as *const K::Value) };
                        return Some(value);
                    }
                    // Continue probing
                }
                None => {
                    return None;
                }
            }
            iter += 1;
            index = (index + iter) % self.capacity;
        }
        return None;
    }

    fn get_mut<K: Key>(&self) -> Option<&mut K::Value> {
        if self.capacity == 0 {
            return None;
        }

        let type_id = TypeId::of::<K>();
        // Todo: Use faster hash
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = (hash % (self.capacity as u64)) as usize;
        let mut iter = 0;
        while iter < self.capacity {
            let entry = unsafe { *self.ptr.add(index) };
            match entry {
                Some((key, ptr, _)) => {
                    if key.eq(&type_id) {
                        let value = unsafe { &mut *(ptr as *mut K::Value) };
                        return Some(value);
                    }
                    // Continue probing
                }
                None => {
                    return None;
                }
            }
            iter += 1;
            index = (index + iter) % self.capacity;
        }
        return None;
    }

    fn insert<K: Key>(&mut self, value: K::Value) {
        if self.capacity == 0 {
            self.grow();
        }

        let type_id = TypeId::of::<K>();
        // Todo: Use faster hash
        let mut hasher = DefaultHasher::new();
        type_id.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = (hash % (self.capacity as u64)) as usize;
        let mut iter = 0;
        while iter < self.capacity {
            let entry = unsafe { &mut *self.ptr.add(index) };
            match entry {
                Some((key, ptr, _)) => {
                    if (*key).eq(&type_id) {
                        if size_of::<K::Value>() > 0 {
                            // Remove old value from heap
                            unsafe {
                                // Drop old value
                                ptr::drop_in_place(*ptr as *mut K::Value);
                                // Dealloc
                                alloc::dealloc(*ptr as *mut u8, Layout::new::<K::Value>());
                            }
                            *ptr = move_to_heap(value) as *const ();
                        }
                        break;
                    }
                }
                None => {
                    let _ = entry.insert((
                        type_id,
                        move_to_heap(value) as *const (),
                        Layout::new::<K::Value>(),
                    ));
                    self.length += 1;
                    // Ensure we have no more than 7/8 of the array used
                    if ((self.capacity > 4) & (self.length > self.capacity / 8 * 7))
                        | ((self.capacity == 4) & (self.length == 4))
                    {
                        self.grow()
                    }
                    break;
                }
            }
            iter += 1;
            index = (index + iter) % self.capacity;
        }
    }
}

fn move_to_heap<T>(value: T) -> *const T {
    if size_of::<T>() == 0 {
        &value as *const T
    } else {
        let ptr = unsafe { alloc::alloc(Layout::new::<T>()) as *mut T };
        unsafe { ptr.write(value) };
        ptr
    }
}

impl Drop for RawTypeContainer {
    fn drop(&mut self) {
        if self.capacity != 0 {
            // Iterate over array, dropping and deallocating
            for i in 0..self.capacity {
                if let Some((_, ptr, layout)) = unsafe { &mut *self.ptr.add(i) } {
                    unsafe {
                        if layout.size() > 0 {
                            ptr::drop_in_place(ptr);
                            alloc::dealloc(*ptr as *mut u8, *layout);
                        }
                    }
                }
            }

            // Deallocate the array
            unsafe {
                alloc::dealloc(
                    self.ptr as *mut u8,
                    Layout::array::<Option<TypeEntry>>(self.capacity).unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_suite;
    use std::{
        alloc::{self, Layout},
        ptr,
    };

    #[test]
    fn test() {
        let ptr = unsafe { alloc::alloc(Layout::array::<*const ()>(2).unwrap()) as *mut *const () };
        let a = 10 as u64;
        let b = true;

        let a_ptr = unsafe { alloc::alloc(Layout::new::<u64>()) as *mut u64 };
        let b_ptr = unsafe { alloc::alloc(Layout::new::<bool>()) as *mut bool };
        //let a_ptr = &*Box::new(10 as u64) as *const u64;
        //let b_ptr = &*Box::new(true) as *const bool;

        unsafe { ptr::write(a_ptr, a) }
        unsafe { ptr::write(b_ptr, b) }

        unsafe {
            ptr::write::<*const ()>(ptr, a_ptr as *const ());
            ptr::write::<*const ()>(ptr.add(1), b_ptr as *const ());
        }

        let first_ref = unsafe { &*(ptr.read() as *const u64) };
        let second_ref = unsafe { &*(ptr.add(1).read() as *const bool) };

        assert_eq!(*first_ref, 10);
        assert_eq!(*second_ref, true);

        unsafe {
            alloc::dealloc(a_ptr as *mut u8, Layout::new::<u64>());
            alloc::dealloc(b_ptr as *mut u8, Layout::new::<bool>());
            alloc::dealloc(ptr as *mut u8, Layout::array::<*const ()>(2).unwrap());
        }
    }

    test_suite!(TypeContainer);
}
