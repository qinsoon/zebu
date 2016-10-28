extern crate mu;
extern crate log;
extern crate simple_logger;
extern crate libloading;

use self::mu::ast::types::*;
use self::mu::ast::ir::*;
use self::mu::ast::inst::*;
use self::mu::ast::op::*;
use self::mu::vm::*;
use self::mu::testutil;

use std::sync::RwLock;
use std::sync::Arc;
use mu::testutil::aot;

#[test]
fn test_udiv() {
    let lib = testutil::compile_fnc("udiv", &udiv);

    unsafe {
        let udiv : libloading::Symbol<unsafe extern fn(u64, u64) -> u64> = lib.get(b"udiv").unwrap();

        let udiv_8_2 = udiv(8, 2);
        println!("udiv(8, 2) = {}", udiv_8_2);
        assert!(udiv_8_2 == 4);
    }
}

fn udiv() -> VM {
    let vm = VM::new();

    // .typedef @int64 = int<64>
    let type_def_int64 = vm.declare_type(vm.next_id(), MuType_::int(64));
    vm.set_name(type_def_int64.as_entity(), Mu("int64"));

    // .funcsig @udiv_sig = (@int64 @int64) -> (@int64)
    let udiv_sig = vm.declare_func_sig(vm.next_id(), vec![type_def_int64.clone()], vec![type_def_int64.clone(), type_def_int64.clone()]);
    vm.set_name(udiv_sig.as_entity(), Mu("udiv_sig"));

    // .funcdecl @udiv <@udiv_sig>
    let func_id = vm.next_id();
    let func = MuFunction::new(func_id, udiv_sig.clone());
    vm.set_name(func.as_entity(), Mu("udiv"));
    vm.declare_func(func);

    // .funcdef @udiv VERSION @udiv_v1 <@udiv_sig>
    let mut func_ver = MuFunctionVersion::new(vm.next_id(), func_id, udiv_sig.clone());
    vm.set_name(func_ver.as_entity(), Mu("udiv_v1"));

    // %entry(<@int64> %a, <@int64> %b):
    let mut blk_entry = Block::new(vm.next_id());
    vm.set_name(blk_entry.as_entity(), Mu("entry"));

    let blk_entry_a = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_a.as_entity(), Mu("blk_entry_a"));
    let blk_entry_b = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_b.as_entity(), Mu("blk_entry_b"));

    // %r = UDIV %a %b
    let blk_entry_r = func_ver.new_ssa(vm.next_id(), type_def_int64.clone());
    vm.set_name(blk_entry_r.as_entity(), Mu("blk_entry_r"));
    let blk_entry_add = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: Some(vec![blk_entry_r.clone_value()]),
        ops: RwLock::new(vec![blk_entry_a.clone(), blk_entry_b.clone()]),
        v: Instruction_::BinOp(BinOp::Udiv, 0, 1)
    });

    // RET %r
    let blk_entry_term = func_ver.new_inst(Instruction{
        hdr: MuEntityHeader::unnamed(vm.next_id()),
        value: None,
        ops: RwLock::new(vec![blk_entry_r.clone()]),
        v: Instruction_::Return(vec![0])
    });

    blk_entry.content = Some(BlockContent{
        args: vec![blk_entry_a.clone_value(), blk_entry_b.clone_value()],
        exn_arg: None,
        body: vec![blk_entry_add, blk_entry_term],
        keepalives: None
    });

    func_ver.define(FunctionContent{
        entry: blk_entry.id(),
        blocks: hashmap!{
            blk_entry.id() => blk_entry
        }
    });

    vm.define_func_version(func_ver);

    vm
}