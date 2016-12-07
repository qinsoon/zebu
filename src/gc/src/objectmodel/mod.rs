use std::sync::atomic;
use utils::{Address, ObjectReference};
use utils::{LOG_POINTER_SIZE, POINTER_SIZE};
use utils::bit_utils;

pub const OBJECT_HEADER_SIZE : usize = 0;

pub static INIT_MARK_STATE : usize = 1;
static MARK_STATE : atomic::AtomicUsize = atomic::ATOMIC_USIZE_INIT;

pub fn init() {
    MARK_STATE.store(INIT_MARK_STATE, atomic::Ordering::SeqCst);
}

pub fn flip_mark_state() {
    let mark_state = MARK_STATE.load(atomic::Ordering::SeqCst);
    MARK_STATE.store(mark_state ^ 1, atomic::Ordering::SeqCst);
}

pub fn load_mark_state() -> u8 {
    MARK_STATE.load(atomic::Ordering::SeqCst) as u8
}

pub fn flip(mark: u8) -> u8 {
    mark ^ 1
}

#[allow(unused_variables)]
pub fn print_object(obj: Address, space_start: Address, trace_map: *mut u8, alloc_map: *mut u8) {
    let mut cursor = obj;
    trace!("OBJECT 0x{:x}", obj);
    loop {
        let hdr = get_ref_byte(alloc_map, space_start, unsafe {cursor.to_object_reference()});
        let (ref_bits, short_encode) = (
            bit_utils::lower_bits(hdr, REF_BITS_LEN),
            bit_utils::test_nth_bit(hdr, SHORT_ENCODE_BIT)
        );


        trace!("0x{:x} | val: 0x{:15x} | {}, hdr: {:b}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 0), hdr);
        cursor = cursor.plus(POINTER_SIZE);
        trace!("0x{:x} | val: 0x{:15x} | {}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 1));

        cursor = cursor.plus(POINTER_SIZE);
        trace!("0x{:x} | val: 0x{:15x} | {}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 2));

        cursor = cursor.plus(POINTER_SIZE);
        trace!("0x{:x} | val: 0x{:15x} | {}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 3));

        cursor = cursor.plus(POINTER_SIZE);
        trace!("0x{:x} | val: 0x{:15x} | {}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 4));

        cursor = cursor.plus(POINTER_SIZE);
        trace!("0x{:x} | val: 0x{:15x} | {} {}",
            cursor, unsafe{cursor.load::<u64>()}, interpret_hdr_for_print_object(hdr, 5), 
            {
                if !short_encode {
                    "MORE..."
                } else {
                    ""
                }
            });
        
        if short_encode {
            return;
        }
    }
}

// index between 0 and 5
fn interpret_hdr_for_print_object(hdr: u8, index: usize) -> &'static str {
    if bit_utils::test_nth_bit(hdr, index) {
        "REF    "
    } else {
        "NON-REF"
    }
}

#[inline(always)]
pub fn mark_as_traced(trace_map: *mut u8, space_start: Address, obj: ObjectReference, mark_state: u8) {
    unsafe {
        *trace_map.offset((obj.to_address().diff(space_start) >> LOG_POINTER_SIZE) as isize) = mark_state;
    }
}

#[inline(always)]
pub fn mark_as_untraced(trace_map: *mut u8, space_start: Address, addr: Address, mark_state: u8) {
    unsafe {
        *trace_map.offset((addr.diff(space_start) >> LOG_POINTER_SIZE) as isize) = mark_state ^ 1;
    }
}

#[inline(always)]
pub fn is_traced(trace_map: *mut u8, space_start: Address, obj: ObjectReference, mark_state: u8) -> bool {
    unsafe {
        (*trace_map.offset((obj.to_address().diff(space_start) >> LOG_POINTER_SIZE) as isize)) == mark_state
    }
}

pub const REF_BITS_LEN    : usize = 6;
pub const OBJ_START_BIT   : usize = 6;
pub const SHORT_ENCODE_BIT : usize = 7;

#[inline(always)]
pub fn get_ref_byte(alloc_map:*mut u8, space_start: Address, obj: ObjectReference) -> u8 {
    unsafe {*alloc_map.offset((obj.to_address().diff(space_start) >> LOG_POINTER_SIZE) as isize)}
}