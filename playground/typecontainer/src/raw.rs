use std::{
    alloc::{self, Layout},
    any::TypeId,
    cell::UnsafeCell,
    hash::{DefaultHasher, Hash, Hasher},
    ptr,
};

use crate::Key;

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
mod test {
    use super::*;
    use crate::test_suite;

    test_suite!(TypeContainer);

    // pub struct Context {
    //     plugin_data: TypeContainer,
    //     initializing_set: RefCell<HashSet<TypeId>>,
    // }

    // impl Context {
    //     fn new() -> Context {
    //         Context {
    //             plugin_data: TypeContainer::new(),
    //             initializing_set: RefCell::new(HashSet::new()),
    //         }
    //     }

    //     fn get_data<T: DataPlugin>(&self, _plugin: T) -> &T::Container {
    //         if let Some(data) = self.plugin_data.get::<T>() {
    //             return data;
    //         } else {
    //             // Initialize the data plugin
    //             let type_id = TypeId::of::<T>();
    //             if self.initializing_set.borrow_mut().insert(type_id) {
    //                 let new_data = T::init(self);
    //                 self.initializing_set.borrow_mut().remove(&type_id);
    //                 let _ = self.plugin_data.try_insert::<T>(new_data);
    //                 return self.plugin_data.get::<T>().unwrap();
    //             } else {
    //                 panic!("Circular dependency detected");
    //             }
    //         }
    //     }

    //     fn get_data_mut<T: DataPlugin>(&mut self, _plugin: T) -> &mut T::Container {
    //         let mut self_shadow = self;
    //         // Use polonius to address borrow checker limitations
    //         polonius!(|self_shadow| -> &'polonius mut T::Container {
    //             if let Some(data) = self_shadow.plugin_data.get_mut::<T>() {
    //                 polonius_return!(data)
    //             }
    //         });
    //         // Initialize the data plugin
    //         let type_id = TypeId::of::<T>();
    //         if self_shadow.initializing_set.borrow_mut().insert(type_id) {
    //             let data = T::init(self_shadow);
    //             self_shadow.initializing_set.borrow_mut().remove(&type_id);
    //             let _ = self_shadow.plugin_data.try_insert::<T>(data);
    //             self_shadow.plugin_data.get_mut::<T>().unwrap()
    //         } else {
    //             panic!("Circular dependency detected");
    //         }
    //     }
    // }

    // pub trait DataPlugin: 'static {
    //     type Container;

    //     fn init(context: &Context) -> Self::Container;
    // }

    // impl<T: DataPlugin> Key for T {
    //     type Value = T::Container;
    // }

    // struct PluginA;
    // impl DataPlugin for PluginA {
    //     type Container = usize;

    //     fn init(context: &Context) -> Self::Container {
    //         if *context.get_data(PluginB) { 1 } else { 0 }
    //     }
    // }

    // struct PluginB;
    // impl DataPlugin for PluginB {
    //     type Container = bool;

    //     fn init(_context: &Context) -> Self::Container {
    //         true
    //     }
    // }

    // #[test]
    // fn test_context_data() {
    //     let context = Context::new();
    //     assert_eq!(*context.get_data(PluginA), 1);
    //     assert!(*context.get_data(PluginB));

    //     let mut context = Context::new();
    //     *context.get_data_mut(PluginB) = false;
    //     assert_eq!(*context.get_data(PluginA), 0);
    // }

    // struct CircularPlugin;
    // impl DataPlugin for CircularPlugin {
    //     type Container = bool;

    //     fn init(context: &Context) -> Self::Container {
    //         *context.get_data(CircularPlugin)
    //     }
    // }

    // #[test]
    // #[should_panic(expected = "Circular dependency detected")]
    // fn test_circular_dep() {
    //     let context = Context::new();
    //     context.get_data(CircularPlugin);
    // }

    // #[test]
    // #[should_panic(expected = "Circular dependency detected")]
    // fn test_circular_dep_mut() {
    //     let mut context = Context::new();
    //     context.get_data_mut(CircularPlugin);
    // }
}
