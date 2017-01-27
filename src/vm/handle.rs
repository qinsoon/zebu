use ast::ir::*;
use ast::ptr::*;
use ast::types::*;

use utils::BitSize;
use utils::Address;

pub type APIHandleResult = Box<APIHandle>;
pub type APIHandleArg<'a>    = &'a APIHandle;

#[derive(Clone, Debug)]
pub struct APIHandle {
    pub id: MuID,
    pub v: APIHandleValue
}

#[derive(Clone, Debug)]
pub enum APIHandleValue {
    Int(u64, BitSize),
    Float(f32),
    Double(f64),
    UPtr(P<MuType>, Address),  // uptr<T>
    UFP (P<MuType>, Address),  // ufuncptr<sig>

    // SeqValue
    Struct(Vec<APIHandleValue>),
    Array (Vec<APIHandleValue>),
    Vector(Vec<APIHandleValue>),

    // GenRef
    Ref (P<MuType>, Address),   // referenced type
    IRef(P<MuType>, Address),
    TagRef64(u64),
    FuncRef,
    ThreadRef,
    StackRef,
    FCRef, // frame cursor ref

    // GenRef->IR
    Bundle,

    // GenRef->IR->Child
    Type(MuID),
    FuncSig(MuID),
    FuncVer(MuID),
    BB,
    Inst,

    // GenRef->IR->Child->Var->Global
    Global(MuID),
    Func(MuID),
    ExpFunc,

    // GenRef->IR->Child->Var->Local
    NorParam,
    ExcParam,
    InstRes,
}

impl APIHandleValue {
    pub fn as_ref_or_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr)
            | &APIHandleValue::IRef(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected Ref or IRef handle")
        }
    }

    pub fn as_ref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::Ref(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected Ref handle")
        }
    }

    pub fn as_iref(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::IRef(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected IRef handle")
        }
    }

    pub fn as_int(&self) -> u64 {
        match self {
            &APIHandleValue::Int(val, _) => val,
            _ => panic!("expected Int handle")
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            &APIHandleValue::Float(val) => val,
            _ => panic!("expected Float handle")
        }
    }

    pub fn as_double(&self) -> f64 {
        match self {
            &APIHandleValue::Double(val) => val,
            _ => panic!("expected Double handle")
        }
    }

    pub fn as_uptr(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UPtr(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UPtr handle")
        }
    }

    pub fn as_ufp(&self) -> (P<MuType>, Address) {
        match self {
            &APIHandleValue::UFP(ref ty, addr) => (ty.clone(), addr),
            _ => panic!("expected UFP handle")
        }
    }

    pub fn as_func(&self) -> MuID {
        match self {
            &APIHandleValue::Func(id) => id,
            _ => panic!("expected FuncRef")
        }
    }
}