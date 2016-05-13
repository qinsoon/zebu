#![allow(dead_code)]
#![allow(non_upper_case_globals)]

pub mod inst_sel;

mod codegen;
pub use compiler::backend::x86_64::codegen::CodeGenerator;

mod asm_backend;
pub use compiler::backend::x86_64::asm_backend::ASMCodeGen;

use ast::ptr::P;
use ast::ir::*;
use ast::types::*;

macro_rules! GPR {
    ($name: expr, $id: expr) => {
        P(Value {
            tag: $name,
            ty: GPR_TY.clone(),
            v: Value_::SSAVar($id)
        })
    };
}

macro_rules! FPR {
    ($name: expr, $id: expr) => {
        P(Value {
            tag: $name,
            ty: FPR_TY.clone(),
            v: Value_::SSAVar($id)
        })
    };
}

lazy_static! {
    pub static ref GPR_TY : P<MuType> = P(MuType::int(64));
    pub static ref FPR_TY : P<MuType> = P(MuType::double());
}

// put into several segments to avoid 'recursion limit reached' error
lazy_static! {
    pub static ref RAX : P<Value> = GPR!("rax", 0);
    pub static ref RCX : P<Value> = GPR!("rcx", 1);
    pub static ref RDX : P<Value> = GPR!("rdx", 2);
    pub static ref RBX : P<Value> = GPR!("rbx", 3);
    pub static ref RSP : P<Value> = GPR!("rsp", 4);
    pub static ref RBP : P<Value> = GPR!("rbp", 5);
    pub static ref RSI : P<Value> = GPR!("rsi", 6);
    pub static ref RDI : P<Value> = GPR!("rdi", 7);
    pub static ref R8  : P<Value> = GPR!("r8",  8);
    pub static ref R9  : P<Value> = GPR!("r9",  9);
    pub static ref R10 : P<Value> = GPR!("r10", 10);
    pub static ref R11 : P<Value> = GPR!("r11", 11);
    pub static ref R12 : P<Value> = GPR!("r12", 12);
    pub static ref R13 : P<Value> = GPR!("r13", 13);
    pub static ref R14 : P<Value> = GPR!("r14", 14);
    pub static ref R15 : P<Value> = GPR!("r15", 15);
    
    pub static ref RETURN_GPRs : [P<Value>; 2] = [
        RAX.clone(),
        RDX.clone(),
    ];
    
    pub static ref ARGUMENT_GPRs : [P<Value>; 6] = [
        RDI.clone(),
        RSI.clone(),
        RDX.clone(),
        RCX.clone(),
        R8.clone(),
        R9.clone()
    ];
    
    pub static ref CALLEE_SAVED_GPRs : [P<Value>; 6] = [
        RBX.clone(),
        RBP.clone(),
        R12.clone(),
        R13.clone(),
        R14.clone(),
        R15.clone()
    ];
}

lazy_static!{
    pub static ref XMM0  : P<Value> = FPR!("xmm0", 20);
    pub static ref XMM1  : P<Value> = FPR!("xmm1", 21);
    pub static ref XMM2  : P<Value> = FPR!("xmm2", 22);
    pub static ref XMM3  : P<Value> = FPR!("xmm3", 23);
    pub static ref XMM4  : P<Value> = FPR!("xmm4", 24);
    pub static ref XMM5  : P<Value> = FPR!("xmm5", 25);
    pub static ref XMM6  : P<Value> = FPR!("xmm6", 26);
    pub static ref XMM7  : P<Value> = FPR!("xmm7", 27);
    pub static ref XMM8  : P<Value> = FPR!("xmm8", 28);
    pub static ref XMM9  : P<Value> = FPR!("xmm9", 29);
    pub static ref XMM10 : P<Value> = FPR!("xmm10",30);
    pub static ref XMM11 : P<Value> = FPR!("xmm11",31);
    pub static ref XMM12 : P<Value> = FPR!("xmm12",32);
    pub static ref XMM13 : P<Value> = FPR!("xmm13",33);
    pub static ref XMM14 : P<Value> = FPR!("xmm14",34);
    pub static ref XMM15 : P<Value> = FPR!("xmm15",35); 
    
    pub static ref RETURN_FPRs : [P<Value>; 2] = [
        XMM0.clone(),
        XMM1.clone()
    ];
    
    pub static ref ARGUMENT_FPRs : [P<Value>; 6] = [
        XMM2.clone(),
        XMM3.clone(),
        XMM4.clone(),
        XMM5.clone(),
        XMM6.clone(),
        XMM7.clone()
    ];
    
    pub static ref CALLEE_SAVED_FPRs : [P<Value>; 0] = [];
}

pub fn is_valid_x86_imm(op: &P<Value>) -> bool {
    use std::u32;
    match op.v {
        Value_::Constant(Constant::Int(val)) if val <= u32::MAX as usize => {
            true
        },
        _ => false
    }
}