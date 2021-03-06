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

extern crate libloading;
extern crate log;

use mu::ast::inst::*;
use mu::ast::ir::*;
use mu::ast::op::*;
use mu::ast::types::*;
use mu::compiler::*;
use mu::utils::LinkedHashMap;
use mu::vm::*;

use mu::linkutils::aot;
use std::sync::Arc;

#[test]
fn test_exception_throw_catch_simple() {
    build_and_run_test!(catch_exception AND throw_exception,
                        catch_exception_test1, throw_catch_simple);
}

fn declare_commons(vm: &VM) {
    typedef!    ((vm) int64      = mu_int(64));
    typedef!    ((vm) ref_int64  = mu_ref(int64));
    typedef!    ((vm) iref_int64 = mu_iref(int64));

    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
}

fn throw_catch_simple() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    let exc_func = create_throw_exception_func(&vm);
    create_catch_exception_func(&vm, true, exc_func);

    vm
}

fn create_catch_exception_func(vm: &VM, use_exception_arg: bool, throw_exception: MuEntityHeader) {
    // .typedef @funcref_throw_exception <@throw_exception_sig>
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));

    typedef!        ((vm) funcref_throw_exception = mu_funcref(throw_exception_sig));
    constdef!       ((vm) <funcref_throw_exception> const_funcref_throw_exception
        = Constant::FuncRef(throw_exception));

    funcsig!        ((vm) catch_exception_sig = () -> ());
    funcdecl!       ((vm) <catch_exception_sig> catch_exception);
    funcdef!        ((vm) <catch_exception_sig> catch_exception VERSION catch_exception_v1);

    // %blk_0():
    block!          ((vm, catch_exception_v1) blk_0);

    block!          ((vm, catch_exception_v1) blk_normal_cont);
    block!          ((vm, catch_exception_v1) blk_exn_cont);

    consta!         ((vm, catch_exception_v1) const_funcref_throw_exception_local
        = const_funcref_throw_exception);
    inst!           ((vm, catch_exception_v1) blk_0_term:
        CALL (const_funcref_throw_exception_local) FUNC(0) (vec![]) CallConvention::Mu,
            normal: blk_normal_cont (vec![]),
            exc   : blk_exn_cont    (vec![])
    );

    define_block!   ((vm, catch_exception_v1) blk_0() {
        blk_0_term
    });

    // %blk_normal_cont():
    inst!           ((vm, catch_exception_v1) blk_normal_cont_threadexit:
        THREADEXIT
    );

    define_block!   ((vm, catch_exception_v1) blk_normal_cont() {
        blk_normal_cont_threadexit
    });

    // %blk_exn_cont() %EXN:
    let ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    ssa!            ((vm, catch_exception_v1) <ref_int64> exn_arg);
    inst!           ((vm, catch_exception_v1) blk_exn_cont_threadexit:
        THREADEXIT
    );

    if use_exception_arg {
        define_block!((vm, catcH_exception_v1) blk_exn_cont() [exn_arg] {
            blk_exn_cont_threadexit
        });
    } else {
        define_block!((vm, catch_exception_v1) blk_exn_cont() {
            blk_exn_cont_threadexit
        });
    }

    define_func_ver!((vm) catch_exception_v1 (entry: blk_0) {
        blk_0, blk_normal_cont, blk_exn_cont
    });

    emit_test! ((vm)
        catch_exception, catch_exception_test1, catch_exception_test1_v1,
        catch_exception_sig,
    );
}

fn create_throw_exception_func(vm: &VM) -> MuEntityHeader {
    let int64 = vm.get_type(vm.id_of("int64"));
    let ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    let iref_int64 = vm.get_type(vm.id_of("iref_int64"));

    funcsig!    ((vm) throw_exception_sig = () -> ());
    funcdecl!   ((vm) <throw_exception_sig> throw_exception);
    funcdef!    ((vm) <throw_exception_sig> throw_exception VERSION throw_exception_v1);

    // %blk_0():
    block!      ((vm, throw_exception_v1) blk_0);

    // %exception_obj = NEW <@int64>
    ssa!        ((vm, throw_exception_v1) <ref_int64> exception_obj);
    inst!       ((vm, throw_exception_v1) blk_0_new:
        exception_obj = NEW <int64>
    );

    // %exception_obj_iref = GETIREF <@int64> %exception_obj
    ssa!        ((vm, throw_exception_v1) <iref_int64> exception_obj_iref);
    inst!       ((vm, throw_exception_v1) blk_0_getiref:
        exception_obj_iref = GETIREF exception_obj
    );

    // STORE <@int64> %exception_obj_iref @int64_1
    let const_int64_1 = vm.get_const(vm.id_of("int64_1"));
    consta!     ((vm, throw_exception_v1) int64_1_local = const_int64_1);
    inst!       ((vm, throw_exception_v1) blk_0_store:
        STORE exception_obj_iref int64_1_local (is_ptr: false, order: MemoryOrder::Relaxed)
    );

    // THROW exception_obj
    inst!       ((vm, throw_exception_v1) blk_0_throw:
        THROW exception_obj
    );

    define_block!((vm, throw_exception_v1) blk_0() {
        blk_0_new,
        blk_0_getiref,
        blk_0_store,
        blk_0_throw
    });

    define_func_ver!((vm) throw_exception_v1(entry: blk_0) {
        blk_0
    });

    throw_exception
}

#[test]
fn test_exception_throw_catch_dont_use_exception_arg() {
    build_and_run_test!(catch_exception AND throw_exception,
                        catch_exception_test1, throw_catch_dont_use_exception_arg);
}

fn throw_catch_dont_use_exception_arg() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    let throw_exc = create_throw_exception_func(&vm);
    create_catch_exception_func(&vm, false, throw_exc);

    vm
}

#[test]
fn test_exception_throw_catch_and_add() {
    build_and_run_test!(catch_and_add AND throw_exception,
                        catch_and_add_test1, throw_catch_and_add);
}

fn throw_catch_and_add() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    let throw_exc = create_throw_exception_func(&vm);
    create_catch_exception_and_add(&vm, throw_exc);

    vm
}

fn create_catch_exception_and_add(vm: &VM, throw_exception: MuEntityHeader) {
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));

    let int64 = vm.get_type(vm.id_of("int64"));
    constdef!   ((vm) <int64> int64_0 = Constant::Int(0));
    constdef!   ((vm) <int64> int64_1 = Constant::Int(1));
    constdef!   ((vm) <int64> int64_2 = Constant::Int(2));
    constdef!   ((vm) <int64> int64_3 = Constant::Int(3));
    constdef!   ((vm) <int64> int64_4 = Constant::Int(4));
    constdef!   ((vm) <int64> int64_5 = Constant::Int(5));

    typedef!    ((vm) type_funcref_throw_exception  = mu_funcref(throw_exception_sig));
    constdef!   ((vm) <type_funcref_throw_exception> const_funcref_throw_exception
        = Constant::FuncRef(throw_exception));

    funcsig!    ((vm) catch_exception_sig = () -> (int64));
    funcdecl!   ((vm) <catch_exception_sig> catch_and_add);
    funcdef!    ((vm) <catch_exception_sig> catch_and_add VERSION catch_and_add_v1);

    // blk_entry
    consta!     ((vm, catch_and_add_v1) int0_local = int64_0);
    consta!     ((vm, catch_and_add_v1) int1_local = int64_1);
    consta!     ((vm, catch_and_add_v1) int2_local = int64_2);
    consta!     ((vm, catch_and_add_v1) int3_local = int64_3);
    consta!     ((vm, catch_and_add_v1) int4_local = int64_4);

    block!      ((vm, catch_and_add_v1) blk_entry);
    block!      ((vm, catch_and_add_v1) blk_main);
    inst!       ((vm, catch_and_add_v1) blk_entry_branch:
        BRANCH blk_main (int0_local, int1_local, int2_local, int3_local, int4_local)
    );

    define_block!   ((vm, catch_and_add_v1) blk_entry () {
        blk_entry_branch
    });

    ssa!        ((vm, catch_and_add_v1) <int64> v0);
    ssa!        ((vm, catch_and_add_v1) <int64> v1);
    ssa!        ((vm, catch_and_add_v1) <int64> v2);
    ssa!        ((vm, catch_and_add_v1) <int64> v3);
    ssa!        ((vm, catch_and_add_v1) <int64> v4);

    // blk_main
    consta!     ((vm, catch_and_add_v1) funcref_throw_local = const_funcref_throw_exception);
    block!      ((vm, catch_and_add_v1) blk_normal);
    block!      ((vm, catch_and_add_v1) blk_exception);
    inst!       ((vm, catch_and_add_v1) blk_main_call:
        CALL (funcref_throw_local, v0, v1, v2, v3, v4) FUNC(0) (vec![]) CallConvention::Mu,
            normal: blk_normal (vec![]),
            exc   : blk_exception (vec![
                DestArg::Normal(1),
                DestArg::Normal(2),
                DestArg::Normal(3),
                DestArg::Normal(4),
                DestArg::Normal(5),
            ])
    );
    define_block!   ((vm, catch_and_add_v1) blk_main(v0, v1, v2, v3, v4) {
        blk_main_call
    });

    // blk_normal
    inst!       ((vm, catch_and_add_v1) blk_normal_threadexit:
        THREADEXIT
    );
    define_block!   ((vm, catch_and_add_v1) blk_normal() {
        blk_normal_threadexit
    });

    // blk_exception
    ssa!        ((vm, catch_and_add_v1) <int64> ev0);
    ssa!        ((vm, catch_and_add_v1) <int64> ev1);
    ssa!        ((vm, catch_and_add_v1) <int64> ev2);
    ssa!        ((vm, catch_and_add_v1) <int64> ev3);
    ssa!        ((vm, catch_and_add_v1) <int64> ev4);
    let ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    let iref_int64 = vm.get_type(vm.id_of("iref_int64"));
    ssa!        ((vm, catch_and_add_v1) <ref_int64> exc_arg);

    inst!       ((vm, catch_and_add_v1) blk_exception_px0:
        PRINTHEX ev0
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px1:
        PRINTHEX ev1
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px2:
        PRINTHEX ev2
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px3:
        PRINTHEX ev3
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px4:
        PRINTHEX ev4
    );
    // load and print exc_arg
    ssa!        ((vm, catch_and_add_v1) <iref_int64> exc_iref);
    inst!       ((vm, catch_and_add_v1) blk_exception_getiref:
        exc_iref = GETIREF exc_arg
    );
    ssa!        ((vm, catch_and_add_v1) <int64> exc_val);
    inst!       ((vm, catch_and_add_v1) blk_exception_load_exc:
        exc_val = LOAD exc_iref (is_ptr: false, order: MemoryOrder::SeqCst)
    );
    inst!       ((vm, catch_and_add_v1) blk_exception_px5:
        PRINTHEX exc_val
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res0);
    inst!       ((vm, catch_and_add_v1) blk_exception_add0:
        res0 = BINOP (BinOp::Add) exc_val ev0
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res1);
    inst!       ((vm, catch_and_add_v1) blk_exception_add1:
        res1 = BINOP (BinOp::Add) res0 ev1
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res2);
    inst!       ((vm, catch_and_add_v1) blk_exception_add2:
        res2 = BINOP (BinOp::Add) res1 ev2
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res3);
    inst!       ((vm, catch_and_add_v1) blk_exception_add3:
        res3 = BINOP (BinOp::Add) res2 ev3
    );

    ssa!        ((vm, catch_and_add_v1) <int64> res4);
    inst!       ((vm, catch_and_add_v1) blk_exception_add4:
        res4 = BINOP (BinOp::Add) res3 ev4
    );

    inst!       ((vm, catch_and_add_v1) blk_exception_ret:
        RET (res4)
    );

    define_block!   ((vm, catch_and_add_v1) blk_exception(ev0, ev1, ev2, ev3, ev4) [exc_arg] {
        blk_exception_px0,
        blk_exception_px1,
        blk_exception_px2,
        blk_exception_px3,
        blk_exception_px4,

        blk_exception_getiref,
        blk_exception_load_exc,
        blk_exception_px5,

        blk_exception_add0,
        blk_exception_add1,
        blk_exception_add2,
        blk_exception_add3,
        blk_exception_add4,

        blk_exception_ret
    });

    define_func_ver!((vm) catch_and_add_v1 (entry: blk_entry) {
        blk_entry, blk_main, blk_normal, blk_exception
    });

    emit_test! ((vm)
        catch_and_add, catch_and_add_test1, catch_and_add_test1_v1,
        RET Int,
        EQ,
        catch_exception_sig,
        RET int64(11u64),
    );
}

#[test]
fn test_exception_throw_catch_twice() {
    build_and_run_test!(catch_twice AND throw_exception, catch_twice_test1, throw_catch_twice);
}

fn throw_catch_twice() -> VM {
    let vm = VM::new();

    declare_commons(&vm);

    let throw_exc = create_throw_exception_func(&vm);
    create_catch_twice(&vm, throw_exc);

    vm
}

fn create_catch_twice(vm: &VM, throw_exception: MuEntityHeader) {
    let throw_exception_sig = vm.get_func_sig(vm.id_of("throw_exception_sig"));
    let ref_int64 = vm.get_type(vm.id_of("ref_int64"));
    let iref_int64 = vm.get_type(vm.id_of("iref_int64"));
    let int64 = vm.get_type(vm.id_of("int64"));

    typedef!    ((vm) type_funcref_throw_exception = mu_funcref(throw_exception_sig));
    constdef!   ((vm) <type_funcref_throw_exception> const_funcref_throw_exception
        = Constant::FuncRef(throw_exception));

    funcsig!    ((vm) catch_exception_sig = () -> (int64));
    funcdecl!   ((vm) <catch_exception_sig> catch_twice);
    funcdef!    ((vm) <catch_exception_sig> catch_twice VERSION catch_twice_v1);

    // blk_entry
    block!      ((vm, catch_twice_v1) blk_entry);
    block!      ((vm, catch_twice_v1) blk_normal);
    block!      ((vm, catch_twice_v1) blk_exception1);
    consta!     ((vm, catch_twice_v1) funcref_throw_local = const_funcref_throw_exception);
    inst!       ((vm, catch_twice_v1) blk_entry_call:
        CALL (funcref_throw_local) FUNC(0) (vec![]) CallConvention::Mu,
            normal: blk_normal (vec![]),
            exc   : blk_exception1 (vec![])
    );

    define_block!((vm, catch_twice_v1) blk_entry() {
        blk_entry_call
    });

    // blk_normal
    inst!       ((vm, catch_twice_v1) blk_normal_threadexit:
        THREADEXIT
    );
    define_block!((vm, catch_twice_v1) blk_normal() {
        blk_normal_threadexit
    });

    // blk_exception1
    block!      ((vm, catch_twice_v1) blk_exception2);
    ssa!        ((vm, catch_twice_v1) <ref_int64> exc_arg1);
    inst!       ((vm, catch_twice_v1) blk_exception1_call:
        CALL (funcref_throw_local, exc_arg1) FUNC(0) (vec![]) CallConvention::Mu,
            normal: blk_normal (vec![]),
            exc   : blk_exception2 (vec![DestArg::Normal(1)])
    );
    define_block!((vm, catch_twice_v1) blk_exception1() [exc_arg1] {
        blk_exception1_call
    });

    // blk_exception2
    ssa!        ((vm, catch_twice_v1) <ref_int64> blk_exception2_exc_arg1);
    ssa!        ((vm, catch_twice_v1) <ref_int64> exc_arg2);

    ssa!        ((vm, catch_twice_v1) <iref_int64> blk_exception2_iref_exc1);
    inst!       ((vm, catch_twice_v1) blk_exception2_getiref1:
        blk_exception2_iref_exc1 = GETIREF blk_exception2_exc_arg1
    );
    ssa!        ((vm, catch_twice_v1) <int64> exc_arg1_val);
    inst!       ((vm, catch_twice_v1) blk_exception2_load1:
        exc_arg1_val = LOAD blk_exception2_iref_exc1 (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    ssa!        ((vm, catch_twice_v1) <iref_int64> blk_exception2_iref_exc2);
    inst!       ((vm, catch_twice_v1) blk_exception2_getiref2:
        blk_exception2_iref_exc2 = GETIREF exc_arg2
    );
    ssa!        ((vm, catch_twice_v1) <int64> exc_arg2_val);
    inst!       ((vm, catch_twice_v1) blk_exception2_load2:
        exc_arg2_val = LOAD blk_exception2_iref_exc2 (is_ptr: false, order: MemoryOrder::SeqCst)
    );

    ssa!        ((vm, catch_twice_v1) <int64> res);
    inst!       ((vm, catch_twice_v1) blk_exception2_add:
        res = BINOP (BinOp::Add) exc_arg1_val exc_arg2_val
    );

    inst!       ((vm, catch_twice_v1) blk_exception2_ret:
        RET (res)
    );

    define_block!   ((vm, catch_twice_v1) blk_exception2(blk_exception2_exc_arg1) [exc_arg2] {
        blk_exception2_getiref1,
        blk_exception2_load1,
        blk_exception2_getiref2,
        blk_exception2_load2,

        blk_exception2_add,
        blk_exception2_ret
    });

    define_func_ver!((vm) catch_twice_v1 (entry: blk_entry) {
        blk_entry,
        blk_normal,
        blk_exception1,
        blk_exception2
    });

    emit_test! ((vm)
        catch_twice, catch_twice_test1, catch_twice_test1_v1,
        RET Int,
        EQ,
        catch_exception_sig,
        RET int64(2u64),
    );
}
