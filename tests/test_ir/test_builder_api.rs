#![allow(unused_imports)]
#![allow(dead_code)]
extern crate mu;

extern crate log;
extern crate simple_logger;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::ptr::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::vm::api::*;

use std::mem;
use std::ptr;
use std::ffi::CString;
use std::os::raw::c_char;

#[test]
#[allow(unused_variables)]
fn test_builder_factorial() {
    builder_factorial()
}

fn builder_factorial() {
//    let mvm = MuVM::new();
//    let mvm_ref = unsafe {mvm.as_mut()}.unwrap();
//    let ctx = (mvm_ref.new_context)(mvm);
//    let ctx_ref = unsafe {ctx.as_mut()}.unwrap();
}

#[test]
#[allow(unused_variables)]
fn test_startup_shutdown() {
    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, ptr::null_mut());
        let id2 = ((*b).gen_sym)(b, CString::new("@id2").unwrap().as_ptr());
        let id3 = ((*b).gen_sym)(b, ptr::null_mut());

        ((*b).abort)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}

#[derive(Default)]
struct CStringPool {
    strings: Vec<CString>,
}

impl CStringPool {
    fn get(&mut self, s: &str) -> *const c_char {
        self.strings.push(CString::new(s).unwrap());
        self.strings.last().unwrap().as_ptr()
    }
}


#[test]
#[allow(unused_variables)]
fn test_types_sigs_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i8"));
        let id2 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id3 = ((*b).gen_sym)(b, csp.get("@pi32"));
        let id4 = ((*b).gen_sym)(b, csp.get("@str1"));
        let id5 = ((*b).gen_sym)(b, ptr::null_mut());
        let id6 = ((*b).gen_sym)(b, csp.get("@str2"));
        let id7 = ((*b).gen_sym)(b, csp.get("@pstr2"));

        ((*b).new_type_int)(b, id1, 8);
        ((*b).new_type_int)(b, id2, 32);
        ((*b).new_type_uptr)(b, id3, id2);
        ((*b).new_type_struct)(b, id4, ptr::null_mut(), 0);
        ((*b).new_type_struct)(b, id5, ptr::null_mut(), 0);

        let mut fields = vec![id3, id7];
        ((*b).new_type_struct)(b, id6, fields.as_mut_ptr(), fields.len());
        ((*b).new_type_uptr)(b, id7, id6);
    
        let id8 = ((*b).gen_sym)(b, csp.get("@sig1"));
        let id9 = ((*b).gen_sym)(b, csp.get("@funcptr1"));

        let mut ptys = vec![id1, id2];
        let mut rtys = vec![id3, id7];
        ((*b).new_funcsig)(b, id8,
                           ptys.as_mut_ptr(), ptys.len(),
                           rtys.as_mut_ptr(), rtys.len());
        ((*b).new_type_ufuncptr)(b, id9, id8);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}


#[test]
#[allow(unused_variables)]
fn test_consts_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id2 = ((*b).gen_sym)(b, csp.get("@CONST_I32_42"));

        ((*b).new_type_int)(b, id1, 32);
        ((*b).new_const_int)(b, id2, id1, 42);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}


#[test]
#[allow(unused_variables)]
fn test_function_loading() {
    let mut csp: CStringPool = Default::default();

    unsafe {
        simple_logger::init_with_level(log::LogLevel::Trace).ok();
        
        info!("Starting micro VM...");

        let mvm = mu_fastimpl_new();

        let ctx = ((*mvm).new_context)(mvm);

        let b = ((*ctx).new_ir_builder)(ctx);

        let id1 = ((*b).gen_sym)(b, csp.get("@i32"));
        let id2 = ((*b).gen_sym)(b, csp.get("@i64"));
        let id3 = ((*b).gen_sym)(b, csp.get("@sig"));
        let id4 = ((*b).gen_sym)(b, csp.get("@func"));

        ((*b).new_type_int)(b, id1, 32);
        ((*b).new_type_int)(b, id2, 64);

        let mut ptys = vec![id1];
        let mut rtys = vec![id2];
        ((*b).new_funcsig)(b, id3,
                           ptys.as_mut_ptr(), ptys.len(),
                           rtys.as_mut_ptr(), rtys.len());

        ((*b).new_func)(b, id4, id3);

        ((*b).load)(b);
        ((*ctx).close_context)(ctx);

        info!("Finished.");
    }
}


