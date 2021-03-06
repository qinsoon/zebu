//// Copyright 2017 The Australian National University
////
//// Licensed under the Apache License, Version 2.0 (the "License");
//// you may not use this file except in compliance with the License.
//// You may obtain a copy of the License at
////
////     http://www.apache.org/licenses/LICENSE-2.0
////
//// Unless required by applicable law or agreed to in writing, software
//// distributed under the License is distributed on an "AS IS" BASIS,
//// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//// See the License for the specific language governing permissions and
//// limitations under the License.
//
//extern crate mu_gc as gc;
//extern crate mu_utils as utils;
//
//use self::gc::start_logging_trace;
//use std::ptr;
//use std::fmt;
//
//#[derive(Copy, Clone)]
//struct Node {
//    next: *mut Node,
//    payload: usize
//}
//
//struct LinkedList<'a> {
//    head: *mut Node,
//    tail: *mut Node,
//    len: usize,
//
//    allocator: &'a mut ImmixAllocator
//}
//
//impl<'a> LinkedList<'a> {
//    fn new(allocator: &mut ImmixAllocator) -> LinkedList {
//        LinkedList {
//            head: ptr::null_mut(),
//            tail: ptr::null_mut(),
//            len: 0,
//
//            allocator: allocator
//        }
//    }
//
//    fn add(&mut self, val: usize) {
//        if self.head.is_null() {
//            let node = Node::new(self.allocator, val);
//
//            self.head = node;
//            self.tail = node;
//            self.len = 1;
//        } else {
//            let node = Node::new(self.allocator, val);
//
//            unsafe {
//                (*self.tail).next = node;
//            }
//            self.tail = node;
//            self.len += 1;
//        }
//    }
//
//    fn verify(&self, expect: Vec<usize>) {
//        if self.len != expect.len() {
//            panic!(
//                "Linked List length: {}, expected length: {}",
//                self.len,
//                expect.len()
//            );
//        }
//
//        let mut i = 0;
//        let mut cursor = self.head;
//
//        while cursor != self.tail {
//            println!("-verifying {:?}-", cursor);
//            println!("{:?}", unsafe { *cursor });
//
//            let val = unsafe { (*cursor).payload };
//            let expect_val = expect[i];
//
//            if val != expect_val {
//                panic!(
//                    "Linked List[{}] = {}, expect[{}] = {}",
//                    i,
//                    val,
//                    i,
//                    expect_val
//                );
//            }
//
//            cursor = unsafe { (*cursor).next };
//            i += 1;
//        }
//    }
//}
//
//impl<'a> fmt::Debug for LinkedList<'a> {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        let mut cursor = self.head;
//
//        // non-tail
//        while cursor != self.tail {
//            write!(f, "{:?}, ", unsafe { *cursor }).unwrap();
//            cursor = unsafe { *cursor }.next;
//        }
//
//        write!(f, "{:?}", unsafe { *cursor }).unwrap();
//
//        Ok(())
//    }
//}
//
//use self::gc::heap;
//use self::gc::heap::immix::ImmixAllocator;
//use self::gc::heap::immix::ImmixSpace;
//use self::gc::heap::freelist;
//use self::gc::heap::freelist::FreeListSpace;
//use self::gc::objectmodel;
//use self::utils::{ObjectReference, Address};
//use std::mem::size_of;
//
//#[cfg(feature = "use-sidemap")]
//const NODE_ENCODE: u64 = 0b1100_0001u64;
//#[cfg(not(feature = "use-sidemap"))]
//const NODE_ENCODE: u64 = 0xb000000000000001u64;
//
//impl Node {
//    fn new(mutator: &mut ImmixAllocator, val: usize) -> *mut Node {
//        println!("-allocating Node({})-", val);
//
//        let addr = mutator.alloc(size_of::<Node>(), 8);
//        println!("returns address {}", addr);
//
//        mutator.init_object(addr, NODE_ENCODE);
//
//        let ptr = addr.to_ptr_mut::<Node>();
//        println!("as pointer {:?}", ptr);
//
//        unsafe {
//            (*ptr).payload = val;
//        }
//        println!("result: {:?}", unsafe { *ptr });
//
//        ptr
//    }
//}
//
//impl fmt::Debug for Node {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        write!(f, "Node({})", self.payload)
//    }
//}
//
//const IMMIX_SPACE_SIZE: usize = 40 << 20;
//const LO_SPACE_SIZE: usize = 0 << 20;
//
//#[test]
//fn create_linked_list() {
//    unsafe {
//        heap::gc::set_low_water_mark();
//    }
//
//    start_logging_trace();
//
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 1, true);
//    gc::print_gc_context();
//
//    let mut mutator = gc::new_mutator();
//
//    {
//        let mut linked_list = LinkedList::new(&mut mutator);
//
//        const N: usize = 5;
//
//        for i in 0..N {
//            linked_list.add(i);
//
//            println!("after add: {:?}", linked_list);
//        }
//
//        linked_list.verify((0..N).collect());
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//fn linked_list_heap_dump() {
//    unsafe {
//        heap::gc::set_low_water_mark();
//    }
//
//    start_logging_trace();
//
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 1, true);
//    gc::print_gc_context();
//
//    let mut mutator = gc::new_mutator();
//
//    {
//        let mut linked_list = LinkedList::new(&mut mutator);
//
//        const N: usize = 5;
//
//        for i in 0..N {
//            linked_list.add(i);
//
//            println!("after add: {:?}", linked_list);
//        }
//
//        // check
//        linked_list.verify((0..N).collect());
//
//        // dump heap from head
//        let head_addr = Address::from_mut_ptr(linked_list.head);
//        let heap_dump = gc::persist_heap(vec![head_addr]);
//
//        println!("{:?}", heap_dump);
//    }
//
//    mutator.destroy();
//}
//
//#[test]
//#[ignore]
//// disable this test because it will cause gcbench fail for unknown reason
//fn linked_list_survive_gc() {
//    unsafe {
//        heap::gc::set_low_water_mark();
//    }
//
//    start_logging_trace();
//
//    gc::gc_init(IMMIX_SPACE_SIZE, LO_SPACE_SIZE, 1, true);
//    gc::print_gc_context();
//
//    let mut mutator = gc::new_mutator();
//
//    {
//        let mut linked_list = LinkedList::new(&mut mutator);
//
//        const N: usize = 5;
//
//        for i in 0..N {
//            linked_list.add(i);
//
//            println!("after add: {:?}", linked_list);
//        }
//
//        // check
//        linked_list.verify((0..N).collect());
//
//        // put head as gc root
//        let head_addr = Address::from_mut_ptr(linked_list.head);
//        gc::add_to_root(unsafe { head_addr.to_object_reference() });
//
//        // force gc
//        gc::force_gc();
//
//        // check
//        linked_list.verify((0..N).collect());
//    }
//
//    mutator.destroy();
//}
