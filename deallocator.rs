use std::collections::HashSet;
use std::ptr;

struct Object {
    marked: bool,
    next: *mut Object,
}

impl Object {
    fn new() -> *mut Object {
        let obj = Box::new(Object { marked: false, next: ptr::null_mut() });
        Box::into_raw(obj)
    }
}

struct Heap {
    first_object: *mut Object,
}

impl Heap {
    fn new() -> Heap {
        Heap { first_object: ptr::null_mut() }
    }

    fn allocate(&mut self) -> *mut Object {
        let obj = Object::new();

        // Set the next pointer of the new object to point to the current first object
        unsafe {
            (*obj).next = self.first_object;
        }

        // Set the first object to be the new object
        self.first_object = obj;

        obj
    }

    fn mark(&self, root_set: &mut HashSet<*mut Object>) {
        // Mark objects reachable from the root set
        for obj in root_set.iter() {
            self.mark_object(*obj);
        }
    }

    fn mark_object(&self, obj: *mut Object) {
        // If the object has already been marked, return
        if obj.is_null() || unsafe { (*obj).marked } {
            return;
        }

        // Mark the object as reachable
        unsafe {
            (*obj).marked = true;
        }

        // Recursively mark objects reachable from this object
        self.mark_object(unsafe { (*obj).next });
    }

    fn sweep(&mut self) {
        // Sweep through the heap, deallocating unmarked objects
        let mut current_obj = &mut self.first_object;
        while !(*current_obj).is_null() {
            if unsafe { (**current_obj).marked } {
                // If the object is marked, unmark it for the next cycle
                unsafe {
                    (**current_obj).marked = false;
                }
                current_obj = unsafe { &mut (**current_obj).next };
            } else {
                // If the object is unmarked, deallocate it
                let obj_to_delete = *current_obj;
                *current_obj = unsafe { (*obj_to_delete).next };
                unsafe {
                    drop(Box::from_raw(obj_to_delete));
                }
            }
        }
    }
}
