use runtime::ValueLocation;

use ast::ir;
use ast::ir::*;
use ast::ptr::*;
use ast::types::*;
use compiler::backend::RegGroup;

use std::sync::RwLock;

pub type EntryFuncSig = MuFuncSig;

pub struct RuntimeEntrypoint {
    pub sig: P<MuFuncSig>,
    pub aot: ValueLocation,
    pub jit: RwLock<Option<ValueLocation>>
}

lazy_static! {
    // impl: runtime_x64_macos.c
    // decl: thread.rs
    pub static ref GET_THREAD_LOCAL : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![ADDRESS_TYPE.clone()],
            arg_tys: vec![]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_get_thread_local")),
        jit: RwLock::new(None),
    };
    
    // impl: swap_stack_x64_macos.s
    // decl: thread.rs
    pub static ref SWAP_BACK_TO_NATIVE_STACK : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig{
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![ADDRESS_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_swap_back_to_native_stack")),
        jit: RwLock::new(None),
    };

    // impl/decl: gc/lib.rs
    pub static ref ALLOC_FAST : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![ADDRESS_TYPE.clone()],
            arg_tys: vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_alloc_fast")),
        jit: RwLock::new(None)
    };
    
    // impl/decl: gc/lib.rs
    pub static ref ALLOC_SLOW : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![ADDRESS_TYPE.clone()],
            arg_tys: vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_alloc_slow")),
        jit: RwLock::new(None),
    };

    // impl/decl: gc/lib.rs
    pub static ref ALLOC_LARGE : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![ADDRESS_TYPE.clone()],
            arg_tys: vec![ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_alloc_large")),
        jit: RwLock::new(None)
    };

    // impl/decl: gc/lib.rs
    pub static ref INIT_OBJ : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone(), UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_init_object")),
        jit: RwLock::new(None)
    };

    // impl/decl: gc/lib.rs
    pub static ref INIT_HYBRID : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![ADDRESS_TYPE.clone(), ADDRESS_TYPE.clone(), UINT64_TYPE.clone(), UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_init_hybrid")),
        jit: RwLock::new(None)
    };
    
    // impl/decl: exception.rs
    pub static ref THROW_EXCEPTION : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![ADDRESS_TYPE.clone()]        
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_throw_exception")),
        jit: RwLock::new(None),
    };

    // impl/decl: math.rs
    pub static ref FREM : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig{
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![DOUBLE_TYPE.clone()],
            arg_tys: vec![DOUBLE_TYPE.clone(), DOUBLE_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_frem")),
        jit: RwLock::new(None)
    };

    // impl/decl: mod.rs
    pub static ref PRINT_HEX : RuntimeEntrypoint = RuntimeEntrypoint {
        sig: P(MuFuncSig {
            hdr: MuEntityHeader::unnamed(ir::new_internal_id()),
            ret_tys: vec![],
            arg_tys: vec![UINT64_TYPE.clone()]
        }),
        aot: ValueLocation::Relocatable(RegGroup::GPR, String::from("muentry_print_hex")),
        jit: RwLock::new(None)
    };
}
