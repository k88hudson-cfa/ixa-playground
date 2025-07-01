use std::{
    alloc::{self, Layout},
    any::TypeId,
    cell::UnsafeCell,
    hash::{DefaultHasher, Hash, Hasher},
    ptr,
};

pub trait Key: 'static {
    type Value;
}

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
        // Safety: all container entries are heap allocated and this insertion
        //  cannot invalidate any existing API-exposed shared pointers
        unsafe { (*self.container.get()).try_insert::<K>(value) }
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
            // This can't overflow because we ensure self.capacity <= isize::MAX.
            let new_capacity = 2 * self.capacity;
            let new_layout = Layout::array::<Option<TypeEntry>>(new_capacity).unwrap();
            (new_capacity, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        // Fill with None
        let new_ptr = unsafe { alloc::alloc(new_layout) as *mut Option<TypeEntry> };
        for i in 0..new_capacity {
            unsafe { ptr::write(new_ptr.add(i), None) }
        }

        if self.capacity > 0 {
            // Need to rehash all entries into new array
            for i in 0..self.capacity {
                if let Some((key, value, layout)) = unsafe { ptr::read(self.ptr.add(i)) } {
                    let entry = unsafe { &mut *seek(key, new_ptr, new_capacity) };
                    match entry {
                        // The keys are unique in the existing array
                        Some(_) => unreachable!(),
                        None => {
                            let _ = entry.insert((key, value, layout));
                        }
                    }
                }
            }

            // Deallocate old array
            let old_layout = Layout::array::<Option<TypeEntry>>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    fn grow_maybe(&mut self) {
        // Ensure we have no more than 7/8 of the array used
        if ((self.capacity > 4) & (self.length > (self.capacity / 8 * 7)))
            | ((self.capacity == 4) & (self.length == 4))
        {
            self.grow()
        }
    }

    fn get<K: Key>(&self) -> Option<&K::Value> {
        if self.capacity == 0 {
            return None;
        }
        unsafe {
            let entry = *seek(TypeId::of::<K>(), self.ptr, self.capacity);
            entry.map(|(_, ptr, _)| &*(ptr as *const K::Value))
        }
    }

    fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        if self.capacity == 0 {
            return None;
        }
        unsafe {
            let entry = *seek(TypeId::of::<K>(), self.ptr, self.capacity);
            entry.map(|(_, ptr, _)| &mut *(ptr as *mut K::Value))
        }
    }

    fn try_insert<K: Key>(&mut self, value: K::Value) -> Result<(), String> {
        if self.capacity == 0 {
            self.grow();
        }
        let type_id = TypeId::of::<K>();
        let entry = unsafe { &mut *seek(type_id, self.ptr, self.capacity) };
        match entry {
            Some(_) => return Err("Key already exists".to_string()),
            None => {
                let _ = entry.insert((
                    type_id,
                    move_to_heap(value) as AnyPtr,
                    Layout::new::<K::Value>(),
                ));
                self.length += 1;
                self.grow_maybe();
                return Ok(());
            }
        }
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

unsafe fn seek(
    type_id: TypeId,
    ptr: *mut Option<TypeEntry>,
    capacity: usize,
) -> *mut Option<TypeEntry> {
    let mut probe = QuadraticProbe::new(get_hash_index(&type_id) as usize, capacity);
    loop {
        let entry = unsafe { ptr.add(probe.get_index()) };
        match unsafe { *entry } {
            Some((key, _, _)) => {
                if key.eq(&type_id) {
                    return entry;
                }
            }
            None => {
                return entry;
            }
        }
        probe.increment();
    }
}

fn get_hash_index(type_id: &TypeId) -> u64 {
    let mut hasher = DefaultHasher::new();
    type_id.hash(&mut hasher);
    hasher.finish()
}

struct QuadraticProbe {
    iteration: usize,
    index: usize,
    modulus: usize,
}

impl QuadraticProbe {
    fn new(start: usize, modulus: usize) -> QuadraticProbe {
        QuadraticProbe {
            iteration: 0,
            index: start % modulus,
            modulus,
        }
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn increment(&mut self) {
        self.iteration += 1;
        self.index = (self.index + self.iteration) % self.modulus;
    }
}

fn move_to_heap<T>(value: T) -> *const T {
    if size_of::<T>() == 0 {
        // ZST so any raw pointer will do
        &value as *const T
    } else {
        let ptr = unsafe { alloc::alloc(Layout::new::<T>()) as *mut T };
        unsafe { ptr.write(value) };
        ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A;
    impl Key for A {
        type Value = usize;
    }

    struct B;
    impl Key for B {
        type Value = bool;
    }

    struct C;
    impl Key for C {
        type Value = ();
    }

    struct D;
    impl Key for D {
        type Value = f64;
    }

    #[test]
    fn test_insert() {
        let container = TypeContainer::new();

        assert!(container.get::<A>().is_none());
        assert!(container.get::<B>().is_none());
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<A>(1).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_none());
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<B>(true).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<C>(()).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<D>(1.0).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));
    }

    #[test]
    fn test_mutate() {
        let mut container = TypeContainer::new();

        let _ = container.try_insert::<A>(1);
        let _ = container.try_insert::<B>(true);
        let _ = container.try_insert::<C>(());
        let _ = container.try_insert::<D>(1.0);

        let a = container.get_mut::<A>().unwrap();
        *a = 2;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let b = container.get_mut::<B>().unwrap();
        *b = false;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let c = container.get_mut::<C>().unwrap();
        *c = ();
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let d = container.get_mut::<D>().unwrap();
        *d = 2.0;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&2.0)));
    }

    #[test]
    fn test_reference_insert() {
        let container = TypeContainer::new();

        // Hold on to shared reference while inserting
        let _ = container.try_insert::<A>(1);
        let a = container.get::<A>().unwrap();

        // Force internal array to be resized
        let _ = container.try_insert::<B>(true);
        let _ = container.try_insert::<C>(());
        let _ = container.try_insert::<D>(1.0);
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        // Check heap reference is still valid
        assert_eq!(a, &1);
    }

    #[test]
    fn test_double_insert() {
        let container = TypeContainer::new();
        assert!(container.try_insert::<A>(1).is_ok());
        assert!(container.try_insert::<A>(2).is_err());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
    }
}
