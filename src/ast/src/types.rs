// Copyright 2017 The Australian National University
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ir::*;
use ptr::P;

use utils::vec_utils;
use utils::POINTER_SIZE;

use std;
use std::collections::HashMap;
use std::fmt;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::RwLock;

// some common types that the compiler may use internally
lazy_static! {
    pub static ref ADDRESS_TYPE: P<MuType> = P(MuType::new(
        new_internal_id(),
        MuType_::int(POINTER_SIZE * 8)
    ));
    pub static ref UINT1_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(1)));
    pub static ref UINT8_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(8)));
    pub static ref UINT16_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(16)));
    pub static ref UINT32_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(32)));
    pub static ref UINT64_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(64)));
    pub static ref UINT128_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::int(128)));
    pub static ref FLOAT_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::float()));
    pub static ref DOUBLE_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::double()));
    pub static ref VOID_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::void()));
    pub static ref REF_VOID_TYPE: P<MuType> = P(MuType::new(
        new_internal_id(),
        MuType_::muref(VOID_TYPE.clone())
    ));
    pub static ref IREF_VOID_TYPE: P<MuType> = P(MuType::new(
        new_internal_id(),
        MuType_::iref(VOID_TYPE.clone())
    ));
    pub static ref UPTR_U8_TYPE: P<MuType> = P(MuType::new(
        new_internal_id(),
        MuType_::uptr(UINT8_TYPE.clone())
    ));
    pub static ref UPTR_U64_TYPE: P<MuType> = P(MuType::new(
        new_internal_id(),
        MuType_::uptr(UINT64_TYPE.clone())
    ));
    pub static ref STACKREF_TYPE: P<MuType> = P(MuType::new(new_internal_id(), MuType_::StackRef));
    pub static ref THREADREF_TYPE: P<MuType> =
        P(MuType::new(new_internal_id(), MuType_::ThreadRef));
    pub static ref INTERNAL_TYPES: Vec<P<MuType>> = vec![
        ADDRESS_TYPE.clone(),
        UINT1_TYPE.clone(),
        UINT8_TYPE.clone(),
        UINT16_TYPE.clone(),
        UINT32_TYPE.clone(),
        UINT64_TYPE.clone(),
        UINT128_TYPE.clone(),
        FLOAT_TYPE.clone(),
        DOUBLE_TYPE.clone(),
        FLOAT_TYPE.clone(),
        VOID_TYPE.clone(),
        REF_VOID_TYPE.clone(),
        IREF_VOID_TYPE.clone(),
        STACKREF_TYPE.clone(),
        THREADREF_TYPE.clone(),
        UPTR_U8_TYPE.clone(),
        UPTR_U64_TYPE.clone()
    ];
}

/// clear struct/hybrid maps, called when creating new VM
pub fn init_types() {
    {
        let mut map_lock = STRUCT_TAG_MAP.write().unwrap();
        map_lock.clear();
    }

    {
        let mut map_lock = HYBRID_TAG_MAP.write().unwrap();
        map_lock.clear();
    }
}

/// MuType represents a Mu type
#[derive(Debug)]
pub struct MuType {
    pub hdr: MuEntityHeader,
    pub v: MuType_,
}

rodal_struct!(MuType { hdr, v });

impl PartialEq for MuType {
    fn eq(&self, other: &MuType) -> bool {
        self.v == other.v
    }
    fn ne(&self, other: &MuType) -> bool {
        self.v != other.v
    }
}

impl MuType {
    /// creates a new Mu type
    pub fn new(id: MuID, v: MuType_) -> MuType {
        MuType {
            hdr: MuEntityHeader::unnamed(id),
            v: v,
        }
    }

    pub fn is_tagref64(&self) -> bool {
        match self.v {
            MuType_::Tagref64 => true,
            _ => false,
        }
    }

    pub fn is_stackref(&self) -> bool {
        match self.v {
            MuType_::StackRef => true,
            _ => false,
        }
    }

    pub fn is_funcref(&self) -> bool {
        match self.v {
            MuType_::FuncRef(_) => true,
            _ => false,
        }
    }

    /// is this type struct type?
    pub fn is_struct(&self) -> bool {
        match self.v {
            MuType_::Struct(_) => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self.v {
            MuType_::Void => true,
            _ => false,
        }
    }

    /// is this type hybrid type?
    pub fn is_hybrid(&self) -> bool {
        match self.v {
            MuType_::Hybrid(_) => true,
            _ => false,
        }
    }

    /// is this type an integer type?
    pub fn is_int(&self) -> bool {
        match self.v {
            MuType_::Int(_) => true,
            _ => false,
        }
    }

    /// is this type an integer type of certain width
    pub fn is_int_n(&self, n: usize) -> bool {
        if let Some(width) = self.get_int_length() {
            width == n
        } else {
            false
        }
    }

    /// is this type a floating point type? (float/double)
    pub fn is_fp(&self) -> bool {
        match self.v {
            MuType_::Float | MuType_::Double => true,
            _ => false,
        }
    }

    pub fn is_opaque_reference(&self) -> bool {
        match self.v {
            MuType_::FuncRef(_) | MuType_::StackRef | MuType_::ThreadRef => true,
            _ => false,
        }
    }

    pub fn is_eq_comparable(&self) -> bool {
        self.is_int()
            || self.is_ptr()
            || self.is_iref()
            || self.is_ref()
            || self.is_opaque_reference()
    }

    pub fn is_ult_comparable(&self) -> bool {
        self.is_int() || self.is_ptr() || self.is_iref()
    }

    /// is this type a float type (single-precision floating point)
    pub fn is_float(&self) -> bool {
        match self.v {
            MuType_::Float => true,
            _ => false,
        }
    }

    /// is this type a double type (double-precision floating point)
    pub fn is_double(&self) -> bool {
        match self.v {
            MuType_::Double => true,
            _ => false,
        }
    }

    /// is this type a scalar type?
    pub fn is_scalar(&self) -> bool {
        match self.v {
            MuType_::Int(_)
            | MuType_::Float
            | MuType_::Double
            | MuType_::Ref(_)
            | MuType_::IRef(_)
            | MuType_::WeakRef(_)
            | MuType_::FuncRef(_)
            | MuType_::UFuncPtr(_)
            | MuType_::ThreadRef
            | MuType_::StackRef
            | MuType_::Tagref64
            | MuType_::UPtr(_) => true,
            _ => false,
        }
    }

    /// gets the tag of a struct/hybrid type, returns None if the type is not hybrid/struct
    /// We use tag to resolve recursive types, and maintains a map between tag and struct types
    pub fn get_struct_hybrid_tag(&self) -> Option<MuName> {
        match self.v {
            MuType_::Hybrid(ref name) | MuType_::Struct(ref name) => Some(name.clone()),
            _ => None,
        }
    }

    /// is this type a reference type?
    /// (only reference type, which does not include iref, or other opaque reference types)
    pub fn is_ref(&self) -> bool {
        match self.v {
            MuType_::Ref(_) => true,
            _ => false,
        }
    }

    /// is this type any reference type pointing to the heap? (including ref/iref/weakref)
    pub fn is_heap_reference(&self) -> bool {
        match self.v {
            MuType_::Ref(_) | MuType_::IRef(_) | MuType_::WeakRef(_) => true,
            _ => false,
        }
    }

    /// is this type an internal reference type?
    pub fn is_iref(&self) -> bool {
        match self.v {
            MuType_::IRef(_) => true,
            _ => false,
        }
    }

    /// is a type raw pointer?
    pub fn is_ptr(&self) -> bool {
        match self.v {
            MuType_::UPtr(_) | MuType_::UFuncPtr(_) => true,
            _ => false,
        }
    }

    /// is this type an aggregated type? (consisted of other types)
    pub fn is_aggregate(&self) -> bool {
        match self.v {
            MuType_::Struct(_) | MuType_::Hybrid(_) | MuType_::Array(_, _) => true,
            _ => false,
        }
    }

    /// is this type a type traced by the garbage collector?
    /// Note: An aggregated type is traced if any of its part is traced.
    #[allow(dead_code)]
    pub fn is_traced(&self) -> bool {
        match self.v {
            MuType_::Ref(_) => true,
            MuType_::IRef(_) => true,
            MuType_::WeakRef(_) => true,
            MuType_::Array(ref elem_ty, _) | MuType_::Vector(ref elem_ty, _) => elem_ty.is_traced(),
            MuType_::ThreadRef | MuType_::StackRef | MuType_::Tagref64 => true,
            MuType_::Hybrid(ref tag) => {
                let map = HYBRID_TAG_MAP.read().unwrap();
                let hybrid_ty = map.get(tag).unwrap();

                let ref fix_tys = hybrid_ty.fix_tys;
                let ref var_ty = hybrid_ty.var_ty;

                var_ty.is_traced()
                    || fix_tys
                        .into_iter()
                        .map(|ty| ty.is_traced())
                        .fold(false, |ret, this| ret || this)
            }
            MuType_::Struct(ref tag) => {
                let map = STRUCT_TAG_MAP.read().unwrap();
                let struct_ty = map.get(tag).unwrap();
                let ref field_tys = struct_ty.tys;

                field_tys
                    .into_iter()
                    .map(|ty| ty.is_traced())
                    .fold(false, |ret, this| ret || this)
            }
            _ => false,
        }
    }

    /// is this type native safe?
    /// Note: An aggregated type is native safe if all of its parts are native safe.
    #[allow(dead_code)]
    pub fn is_native_safe(&self) -> bool {
        match self.v {
            MuType_::Int(_) => true,
            MuType_::Float => true,
            MuType_::Double => true,
            MuType_::Void => true,
            MuType_::Array(ref elem_ty, _) | MuType_::Vector(ref elem_ty, _) => {
                elem_ty.is_native_safe()
            }
            MuType_::UPtr(_) => true,
            MuType_::UFuncPtr(_) => true,
            MuType_::Hybrid(ref tag) => {
                let map = HYBRID_TAG_MAP.read().unwrap();
                let hybrid_ty = map.get(tag).unwrap();

                let ref fix_tys = hybrid_ty.fix_tys;
                let ref var_ty = hybrid_ty.var_ty;

                var_ty.is_native_safe()
                    && fix_tys
                        .into_iter()
                        .map(|ty| ty.is_native_safe())
                        .fold(true, |ret, this| ret && this)
            }
            MuType_::Struct(ref tag) => {
                let map = STRUCT_TAG_MAP.read().unwrap();
                let struct_ty = map.get(tag).unwrap();
                let ref field_tys = struct_ty.tys;

                field_tys
                    .into_iter()
                    .map(|ty| ty.is_native_safe())
                    .fold(true, |ret, this| ret && this)
            }
            _ => false,
        }
    }

    /// gets the element type of an array type, returns None if the type is not an array type
    pub fn get_elem_ty(&self) -> Option<P<MuType>> {
        match self.v {
            MuType_::Array(ref elem_ty, _) => Some(elem_ty.clone()),
            _ => None,
        }
    }

    /// gets the signature of a funcref or ufuncptr type
    pub fn get_sig(&self) -> Option<P<MuFuncSig>> {
        match self.v {
            MuType_::FuncRef(ref sig) | MuType_::UFuncPtr(ref sig) => Some(sig.clone()),
            _ => None,
        }
    }

    /// gets a field's type of a struct type,
    /// returns None if the type is not a struct or hybrid type
    pub fn get_field_ty(&self, index: usize) -> Option<P<MuType>> {
        match self.v {
            MuType_::Struct(ref tag) => {
                let map_lock = STRUCT_TAG_MAP.read().unwrap();
                let struct_inner = map_lock.get(tag).unwrap();

                Some(struct_inner.tys[index].clone())
            }
            MuType_::Hybrid(ref tag) => {
                let map_lock = HYBRID_TAG_MAP.read().unwrap();
                let hybrid_inner = map_lock.get(tag).unwrap();

                Some(hybrid_inner.fix_tys[index].clone())
            }
            _ => None,
        }
    }

    /// gets the var part type of a hybrid type, returns None if the type is not a hybrid type
    pub fn get_hybrid_varpart_ty(&self) -> Option<P<MuType>> {
        match self.v {
            MuType_::Hybrid(ref tag) => {
                let map_lock = HYBRID_TAG_MAP.read().unwrap();
                let hybrid_inner = map_lock.get(tag).unwrap();

                Some(hybrid_inner.var_ty.clone())
            }
            _ => None,
        }
    }

    /// gets the referent type for Ref/IRef/WeakRef/UPtr, returns None if the type is
    /// not any mentioned type.
    pub fn get_referent_ty(&self) -> Option<P<MuType>> {
        use types::MuType_::*;
        match self.v {
            Ref(ref ty) | IRef(ref ty) | WeakRef(ref ty) | UPtr(ref ty) => Some(ty.clone()),
            _ => None,
        }
    }

    /// gets the function signature for FuncRef or UFuncPtr, return None if the type is not
    /// those two types
    pub fn get_func_sig(&self) -> Option<P<MuFuncSig>> {
        match self.v {
            MuType_::FuncRef(ref sig) | MuType_::UFuncPtr(ref sig) => Some(sig.clone()),
            _ => None,
        }
    }

    /// gets the length (in bit) of a integer/pointer type (assume pointer types are always 64 bits)
    // FIXME: should deprecate this function, and get the length from BackendType
    pub fn get_int_length(&self) -> Option<usize> {
        use types::MuType_::*;
        match self.v {
            Int(len) => Some(len),
            Ref(_) | IRef(_) | WeakRef(_) | UPtr(_) | ThreadRef | StackRef | Tagref64
            | FuncRef(_) | UFuncPtr(_) => Some(64),
            _ => None,
        }
    }

    /// prints a struct type
    pub fn print_details(&self) -> String {
        match self.v {
            MuType_::Struct(ref tag) => {
                let lock = STRUCT_TAG_MAP.read().unwrap();
                format!("{} = {}", tag, lock.get(tag).unwrap())
            }
            MuType_::Hybrid(ref tag) => {
                let lock = HYBRID_TAG_MAP.read().unwrap();
                format!("{} = {}", tag, lock.get(tag).unwrap())
            }
            _ => format!("{}", self),
        }
    }

    /// prints a struct type
    pub fn print_hybrid(&self) -> String {
        match self.v {
            _ => panic!(),
        }
    }
}

pub type StructTag = MuName;
pub type HybridTag = MuName;

/// MuType_ is used for pattern matching for MuType
#[derive(PartialEq, Debug, Clone)]
pub enum MuType_ {
    /// int <length>
    Int(usize),
    /// float
    Float,
    /// double
    Double,

    /// ref<T>
    Ref(P<MuType>), // Box is needed for non-recursive enum
    /// iref<T>: internal reference
    IRef(P<MuType>),
    /// weakref<T>
    WeakRef(P<MuType>),

    /// uptr<T>: unsafe pointer
    UPtr(P<MuType>),

    /// struct<T1 T2 ...>
    Struct(StructTag),

    /// array<T length>
    Array(P<MuType>, usize),

    /// hybrid<F1 F2 ... V>: a hybrid of fixed length parts and a variable length part
    Hybrid(HybridTag),

    /// void
    Void,

    /// threadref
    ThreadRef,
    /// stackref
    StackRef,

    /// tagref64: hold a double or an int or an ref<void>
    Tagref64,

    /// vector<T length>
    Vector(P<MuType>, usize),

    /// funcref<@sig>
    FuncRef(P<MuFuncSig>),

    /// ufuncptr<@sig>
    UFuncPtr(P<MuFuncSig>),
}
impl MuType_ {
    pub fn strong_variant(&self) -> MuType_ {
        match self {
            &MuType_::WeakRef(ref t) => MuType_::Ref(t.clone()),
            _ => self.clone(),
        }
    }
}
rodal_enum!(MuType_{(Int: size), Float, Double, (Ref: ty), (IRef: ty), (WeakRef: ty), (UPtr: ty),
    (Struct: tag), (Array: ty, size), (Hybrid: tag), Void, ThreadRef, StackRef, Tagref64,
    (Vector: ty, size), (FuncRef: ty), (UFuncPtr: ty)});

impl fmt::Display for MuType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.v)
    }
}

impl fmt::Display for MuType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &MuType_::Int(n) => write!(f, "int<{}>", n),
            &MuType_::Float => write!(f, "float"),
            &MuType_::Double => write!(f, "double"),
            &MuType_::Ref(ref ty) => write!(f, "ref<{}>", ty),
            &MuType_::IRef(ref ty) => write!(f, "iref<{}>", ty),
            &MuType_::WeakRef(ref ty) => write!(f, "weakref<{}>", ty),
            &MuType_::UPtr(ref ty) => write!(f, "uptr<{}>", ty),
            &MuType_::Array(ref ty, size) => write!(f, "array<{} {}>", ty, size),
            &MuType_::Void => write!(f, "void"),
            &MuType_::ThreadRef => write!(f, "threadref"),
            &MuType_::StackRef => write!(f, "stackref"),
            &MuType_::Tagref64 => write!(f, "tagref64"),
            &MuType_::Vector(ref ty, size) => write!(f, "vector<{} {}>", ty, size),
            &MuType_::FuncRef(ref sig) => write!(f, "funcref<{}>", sig),
            &MuType_::UFuncPtr(ref sig) => write!(f, "ufuncptr<{}>", sig),
            &MuType_::Struct(ref tag) => write!(f, "{}", tag),
            &MuType_::Hybrid(ref tag) => write!(f, "{}", tag),
        }
    }
}

#[no_mangle]
pub static STRUCT_TAG_MAP_LOC: Option<AtomicPtr<RwLock<HashMap<StructTag, StructType_>>>> = None;
#[no_mangle]
pub static HYBRID_TAG_MAP_LOC: Option<AtomicPtr<RwLock<HashMap<HybridTag, HybridType_>>>> = None;

lazy_static! {
    /// storing a map from MuName to StructType_
    pub static ref STRUCT_TAG_MAP : RwLock<HashMap<StructTag, StructType_>> =
        match &STRUCT_TAG_MAP_LOC {
            &Some(ref ptr) => unsafe{ptr::read(ptr.load(Ordering::Relaxed))},
            &None => RwLock::new(HashMap::new())
        };
    /// storing a map from MuName to HybridType_
    pub static ref HYBRID_TAG_MAP : RwLock<HashMap<HybridTag, HybridType_>> =
        match &HYBRID_TAG_MAP_LOC {
            &Some(ref ptr) => unsafe{ptr::read(ptr.load(Ordering::Relaxed))},
            &None => RwLock::new(HashMap::new())
        };
}

rodal_struct!(StructType_ { tys });
#[derive(PartialEq, Debug)]
pub struct StructType_ {
    tys: Vec<P<MuType>>,
}

impl fmt::Display for StructType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "struct<").unwrap();
        for i in 0..self.tys.len() {
            let ty = &self.tys[i];
            write!(f, "{}", ty).unwrap();
            if i != self.tys.len() - 1 {
                write!(f, " ").unwrap();
            }
        }
        write!(f, ">")
    }
}

impl StructType_ {
    // The IR builder needs to create StructType objects, too.
    pub fn new(tys: Vec<P<MuType>>) -> StructType_ {
        StructType_ { tys: tys }
    }
    pub fn set_tys(&mut self, mut list: Vec<P<MuType>>) {
        self.tys.clear();
        self.tys.append(&mut list);
    }

    pub fn get_tys(&self) -> &Vec<P<MuType>> {
        &self.tys
    }
}

rodal_struct!(HybridType_ { fix_tys, var_ty });
#[derive(PartialEq, Debug)]
pub struct HybridType_ {
    fix_tys: Vec<P<MuType>>,
    var_ty: P<MuType>,
}

impl HybridType_ {
    pub fn new(fix_tys: Vec<P<MuType>>, var_ty: P<MuType>) -> HybridType_ {
        HybridType_ {
            fix_tys: fix_tys,
            var_ty: var_ty,
        }
    }

    pub fn set_tys(&mut self, mut fix_tys: Vec<P<MuType>>, var_ty: P<MuType>) {
        self.fix_tys.clear();
        self.fix_tys.append(&mut fix_tys);

        self.var_ty = var_ty;
    }

    pub fn get_fix_tys(&self) -> &Vec<P<MuType>> {
        &self.fix_tys
    }

    pub fn get_var_ty(&self) -> &P<MuType> {
        &self.var_ty
    }
}

impl fmt::Display for HybridType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hybrid<").unwrap();
        for i in 0..self.fix_tys.len() {
            let ty = &self.fix_tys[i];
            write!(f, "{}", ty).unwrap();
            if i != self.fix_tys.len() - 1 {
                write!(f, " ").unwrap();
            }
        }
        write!(f, " {}>", self.var_ty)
    }
}

impl MuType_ {
    pub fn int(len: usize) -> MuType_ {
        MuType_::Int(len)
    }
    pub fn float() -> MuType_ {
        MuType_::Float
    }
    pub fn double() -> MuType_ {
        MuType_::Double
    }
    pub fn muref(referent: P<MuType>) -> MuType_ {
        MuType_::Ref(referent)
    }
    pub fn iref(referent: P<MuType>) -> MuType_ {
        MuType_::IRef(referent)
    }
    pub fn weakref(referent: P<MuType>) -> MuType_ {
        MuType_::WeakRef(referent)
    }
    pub fn uptr(referent: P<MuType>) -> MuType_ {
        MuType_::UPtr(referent)
    }
    pub fn array(ty: P<MuType>, len: usize) -> MuType_ {
        MuType_::Array(ty, len)
    }
    pub fn void() -> MuType_ {
        MuType_::Void
    }
    pub fn threadref() -> MuType_ {
        MuType_::ThreadRef
    }
    pub fn stackref() -> MuType_ {
        MuType_::StackRef
    }
    pub fn tagref64() -> MuType_ {
        MuType_::Tagref64
    }
    pub fn vector(ty: P<MuType>, len: usize) -> MuType_ {
        MuType_::Vector(ty, len)
    }
    pub fn funcref(sig: P<MuFuncSig>) -> MuType_ {
        MuType_::FuncRef(sig)
    }
    pub fn ufuncptr(sig: P<MuFuncSig>) -> MuType_ {
        MuType_::UFuncPtr(sig)
    }

    /// creates an empty struct type with a tag (we can later put types into the struct)
    /// This is used to create a recursive struct type, e.g. T = struct { ref<T> }
    pub fn mustruct_empty(tag: MuName) -> MuType_ {
        let struct_ty_ = StructType_ { tys: vec![] };
        STRUCT_TAG_MAP
            .write()
            .unwrap()
            .insert(tag.clone(), struct_ty_);

        MuType_::Struct(tag)
    }
    /// puts types into an empty struct (created by mustruct_empty())
    /// This method will clear existing types declared with the tag,
    /// and set struct to the specified types
    /// This method panics if the tag does not exist
    pub fn mustruct_put(tag: &MuName, mut list: Vec<P<MuType>>) {
        let mut map_guard = STRUCT_TAG_MAP.write().unwrap();

        match map_guard.get_mut(tag) {
            Some(struct_ty_) => {
                struct_ty_.tys.clear();
                struct_ty_.tys.append(&mut list);
            }
            None => panic!("call mustruct_empty() to create an empty struct before mustruct_put()"),
        }
    }
    /// creates a Mu struct with specified field types
    pub fn mustruct(tag: StructTag, list: Vec<P<MuType>>) -> MuType_ {
        let struct_ty_ = StructType_ { tys: list };

        // if there is an attempt to use a same tag for different struct,
        // we panic
        match STRUCT_TAG_MAP.read().unwrap().get(&tag) {
            Some(old_struct_ty_) => {
                if struct_ty_ != *old_struct_ty_ {
                    panic!(
                        "trying to insert {} as {}, while the old struct is defined as {}",
                        struct_ty_, tag, old_struct_ty_
                    )
                }
            }
            None => {}
        }
        // otherwise, store the tag
        STRUCT_TAG_MAP
            .write()
            .unwrap()
            .insert(tag.clone(), struct_ty_);

        MuType_::Struct(tag)
    }

    /// creates an empty hybrid type with a tag (we can later put types into the hybrid)
    /// This is used to create a recursive hybrid type, e.g. T = hybrid { ref<T>, ... | ref<T> }
    pub fn hybrid_empty(tag: HybridTag) -> MuType_ {
        let hybrid_ty_ = HybridType_ {
            fix_tys: vec![],
            var_ty: VOID_TYPE.clone(),
        };
        HYBRID_TAG_MAP
            .write()
            .unwrap()
            .insert(tag.clone(), hybrid_ty_);

        MuType_::Hybrid(tag)
    }

    /// puts types into an empty hybrid (created by muhybrid_empty())
    /// This method will clear existing types declared with the tag,
    /// and set hybrid to the specified types
    /// This method panics if the tag does not exist
    pub fn hybrid_put(tag: &HybridTag, mut fix_tys: Vec<P<MuType>>, var_ty: P<MuType>) {
        let mut map_guard = HYBRID_TAG_MAP.write().unwrap();

        match map_guard.get_mut(tag) {
            Some(hybrid_ty_) => {
                hybrid_ty_.fix_tys.clear();
                hybrid_ty_.fix_tys.append(&mut fix_tys);

                hybrid_ty_.var_ty = var_ty;
            }
            None => panic!("call hybrid_empty() to create an empty struct before hybrid_put()"),
        }
    }
    /// creates a Mu hybrid with specified fix part and var part types
    pub fn hybrid(tag: HybridTag, fix_tys: Vec<P<MuType>>, var_ty: P<MuType>) -> MuType_ {
        let hybrid_ty_ = HybridType_ {
            fix_tys: fix_tys,
            var_ty: var_ty,
        };

        match HYBRID_TAG_MAP.read().unwrap().get(&tag) {
            Some(old_hybrid_ty_) => {
                if hybrid_ty_ != *old_hybrid_ty_ {
                    panic!(
                        "trying to insert {} as {}, while the old hybrid is defined as {}",
                        hybrid_ty_, tag, old_hybrid_ty_
                    );
                }
            }
            None => {}
        }

        HYBRID_TAG_MAP
            .write()
            .unwrap()
            .insert(tag.clone(), hybrid_ty_);

        MuType_::Hybrid(tag)
    }
}

/// MuFuncSig represents a Mu function signature
#[derive(Debug)]
pub struct MuFuncSig {
    pub hdr: MuEntityHeader,
    pub ret_tys: Vec<P<MuType>>,
    pub arg_tys: Vec<P<MuType>>,
}

impl PartialEq for MuFuncSig {
    fn eq(&self, other: &MuFuncSig) -> bool {
        self.ret_tys == other.ret_tys && self.arg_tys == other.arg_tys
    }
}
rodal_struct!(MuFuncSig {
    hdr,
    ret_tys,
    arg_tys
});

impl fmt::Display for MuFuncSig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({})->({})",
            vec_utils::as_str_sp(&self.arg_tys),
            vec_utils::as_str_sp(&self.ret_tys)
        )
    }
}

pub type CFuncSig = MuFuncSig;
