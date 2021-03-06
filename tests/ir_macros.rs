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

#![allow(unused_macros)]

macro_rules! typedef {
    // int, floating point
    (($vm: expr) $name: ident = mu_int($len: expr)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::int($len));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_double) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::double());
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_float) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::float());
        $vm.set_name($name.as_entity());
    };

    // ref, iref, ptr
    (($vm: expr) $name: ident = mu_ref($ty: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::muref($ty.clone()));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_iref($ty: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::iref($ty.clone()));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_uptr($ty: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::uptr($ty.clone()));
        $vm.set_name($name.as_entity());
    };

    // struct
    (($vm: expr) $name: ident = mu_struct($($ty: ident), *)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::mustruct(Mu(stringify!($name)),
                                                       vec![$($ty.clone()),*]));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_struct()) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::mustruct(Mu(stringify!($name)), vec![]));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) $name: ident = mu_struct_placeholder()) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::mustruct_empty(Mu(stringify!($name))));
        $vm.set_name($name.as_entity());
    };
    (($vm: expr) mu_struct_put($name: ident, $($ty: ident), *)) => {
        MuType_::mustruct_put(&Mu(stringify!($name)), vec![$($ty.clone()), *])
    };

    // hybrid
    (($vm: expr) $name: ident = mu_hybrid($($ty: ident), *)($var_ty: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::hybrid(Mu(stringify!($name)),
                                                     vec![$($ty.clone()), *], $var_ty.clone()));
        $vm.set_name($name.as_entity());
    };

    // array
    (($vm: expr) $name: ident = mu_array($ty: ident, $len: expr)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::array($ty.clone(), $len));
        $vm.set_name($name.as_entity());
    };

    // funcref
    (($vm: expr) $name: ident = mu_funcref($sig: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::funcref($sig.clone()));
        $vm.set_name($name.as_entity());
    };

    // ufuncptr
    (($vm: expr) $name: ident = mu_ufuncptr($sig: ident)) => {
        let $name = $vm.declare_type(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
                                     MuType_::ufuncptr($sig.clone()));
        $vm.set_name($name.as_entity());
    };
}

macro_rules! constdef {
    (($vm: expr) <$ty: ident> $name: ident = $val: expr) => {
        let $name = $vm.declare_const(
            MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
            $ty.clone(),
            $val,
        );
        $vm.set_name($name.as_entity());
    };
}

macro_rules! globaldef {
    (($vm: expr) <$ty: ident> $name: ident) => {
        let $name = $vm.declare_global(
            MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
            $ty.clone(),
        );
        $vm.set_name($name.as_entity());
    };
}

macro_rules! funcsig {
    (($vm: expr) $name: ident = ($($arg_ty: ident),*) -> ($($ret_ty: ident),*)) => {
        let $name = $vm.declare_func_sig(MuEntityHeader::named($vm.next_id(),
                                                               Mu(stringify!($name))),
                                         vec![$($ret_ty.clone()),*], vec![$($arg_ty.clone()),*]);
        $vm.set_name($name.as_entity());
    }
}

macro_rules! funcdecl {
    (($vm: expr) <$sig: ident> $name: ident) => {
        let func = MuFunction::new(
            MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
            $sig.clone(),
        );
        $vm.set_name(func.as_entity());
        let $name = func.hdr.clone();
        $vm.declare_func(func);
    };
}

macro_rules! funcdef {
    (($vm: expr) <$sig: ident> $func: ident VERSION $version: ident) => {
        let mut $version = MuFunctionVersion::new(
            MuEntityHeader::named($vm.next_id(), Mu(stringify!($version))),
            $func.id(),
            $sig.clone(),
        );
        $vm.set_name($version.as_entity());
    };
}

macro_rules! define_func_ver {
    (($vm: expr) $fv: ident (entry: $entry: ident){$($blk: ident), *}) => {
        $fv.define(FunctionContent::new($entry.id(), {
            let mut ret = LinkedHashMap::new();
            $ (ret.insert($blk.id(), $blk); )*
            ret
        }));

        $vm.define_func_version($fv);
    }
}

macro_rules! block {
    (($vm: expr, $fv: ident) $name: ident) => {
        let mut $name = Block::new(MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))));
        $vm.set_name($name.as_entity());
    };
}

macro_rules! define_block {
    (($vm: expr, $fv: ident) $name: ident ($($arg: ident), *) {$($inst: ident), *}) => {
        $name.content = Some(BlockContent{
            args: vec![$($arg.clone_value()), *],
            exn_arg: None,
            body: vec![$($inst), *],
            keepalives: None
        });
    };

    (($vm: expr, $fv: ident) $name: ident ($($arg: ident), *)
     [$exn_arg: ident] {$($inst: ident), *}) => {
        $name.content = Some(BlockContent{
            args: vec![$($arg.clone_value()), *],
            exn_arg: Some($exn_arg.clone_value()),
            body: vec![$($inst), *],
            keepalives: None
        });
    }
}

macro_rules! ssa {
    (($vm: expr, $fv: ident) <$ty: ident> $name: ident) => {
        let $name = $fv.new_ssa(
            MuEntityHeader::named($vm.next_id(), Mu(stringify!($name))),
            $ty.clone(),
        );
        $vm.set_name($name.as_entity());
    };
}

macro_rules! machine_reg {
    (($vm: expr, $fv: ident) $name: ident = $mreg: expr) => {
        let $name = $fv.new_machine_reg($mreg.clone());
    };
}

macro_rules! consta {
    (($vm: expr, $fv: ident) $name: ident = $c: ident) => {
        let $name = $fv.new_constant($c.clone());
    };
}

macro_rules! global {
    (($vm: expr, $fv: ident) $name: ident = $g: ident) => {
        let $name = $fv.new_global($g.clone());
    };
}

macro_rules! inst {
    // NEW
    (($vm: expr, $fv: ident) $name: ident: $value: ident = NEW <$ty: ident>) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![],
            v:      Instruction_::New($ty.clone())
        });
    };

    // NEWHYBRID
    (($vm: expr, $fv: ident) $name: ident: $value: ident = NEWHYBRID <$ty: ident> $len: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$len.clone()],
            v:      Instruction_::NewHybrid($ty.clone(), 0)
        });
    };

    // GETIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETIREF $op: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op.clone()],
            v:      Instruction_::GetIRef(0)
        });
    };

    // GETFIELDIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETFIELDIREF $op: ident
     (is_ptr: $is_ptr: expr, index: $index: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op.clone()],
            v:      Instruction_::GetFieldIRef {
                        is_ptr: $is_ptr,
                        base: 0,
                        index: $index
            }
        });
    };

    // GETELEMIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETELEMIREF $op: ident $index: ident
     (is_ptr: $is_ptr: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op.clone(), $index.clone()],
            v:      Instruction_::GetElementIRef {
                        is_ptr: $is_ptr,
                        base: 0,
                        index: 1
            }
        });
    };

    // GETVARPARTIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = GETVARPARTIREF $op: ident
     (is_ptr: $is_ptr: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op.clone()],
            v:      Instruction_::GetVarPartIRef {
                        is_ptr: $is_ptr,
                        base: 0
            }
        });
    };

    // SHIFTIREF
    (($vm: expr, $fv: ident) $name: ident: $value: ident = SHIFTIREF $op: ident $offset: ident
     (is_ptr: $is_ptr: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op.clone(), $offset.clone()],
            v:      Instruction_::ShiftIRef {
                        is_ptr: $is_ptr,
                        base: 0,
                        offset: 1
            }
        });
    };

    // STORE
    (($vm: expr, $fv: ident) $name: ident: STORE $loc: ident $val: ident
     (is_ptr: $is_ptr: expr, order: $order: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    vec![$loc.clone(), $val.clone()],
            v:      Instruction_::Store {
                        is_ptr: $is_ptr,
                        order: $order,
                        mem_loc: 0,
                        value: 1
            }
        });
    };

    // LOAD
    (($vm: expr, $fv: ident) $name: ident: $value: ident = LOAD $loc: ident
     (is_ptr: $is_ptr: expr, order: $order: expr)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$loc.clone()],
            v:      Instruction_::Load {
                        is_ptr: $is_ptr,
                        order: $order,
                        mem_loc: 0
            }
        });
    };

    // BINOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident =
     BINOP ($op: expr) $op1: ident $op2: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value()]),
            ops:    vec![$op1.clone(), $op2.clone()],
            v:      Instruction_::BinOp($op, 0, 1)
        });
    };

    // BINOP with status
    (($vm: expr, $fv: ident) $name: ident: $value: ident, $($flag: ident), * =
     BINOP_STATUS ($op: expr) ($flags: expr) $op1: ident $op2: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$value.clone_value(), $($flag.clone_value()), *]),
            ops:    vec![$op1.clone(), $op2.clone()],
            v:      Instruction_::BinOpWithStatus($op, $flags, 0, 1)
        });
    };

    // CMPOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident =
     CMPOP ($op: expr) $op1: ident $op2: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: vec![$op1.clone(), $op2.clone()],
            v: Instruction_::CmpOp($op, 0, 1)
        });
    };

    // CONVOP
    (($vm: expr, $fv: ident) $name: ident: $value: ident =
     CONVOP ($operation: expr) <$ty1: ident $ty2: ident> $operand: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: vec![$operand.clone()],
            v: Instruction_::ConvOp{
                operation: $operation,
                from_ty: $ty1.clone(),
                to_ty: $ty2.clone(),
                operand: 0
            }
        });
    };

    // SELECT
    (($vm: expr, $fv: ident) $name: ident: $value: ident =
     SELECT $cond: ident $op_true: ident $op_false:ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$value.clone_value()]),
            ops: vec![$cond.clone(), $op_true.clone(), $op_false.clone()],
            v: Instruction_::Select{
                cond: 0,
                true_val: 1,
                false_val: 2
            }
        });
    };

    // BRANCH
    (($vm: expr, $fv: ident) $name: ident: BRANCH $dest: ident ($($arg: ident), *)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    vec![$($arg.clone()),*],
            v:      Instruction_::Branch1(Destination{
                        target: $dest.hdr.clone(),
                        args: {
                            let mut i =0;
                            vec![$($arg.clone()),*].iter().map(|_: &Arc<TreeNode>| {
                                let ret = DestArg::Normal(i); i+=1; ret
                             }).collect()
                        }
            })
        });
    };

    // BRANCH2
    // list all operands first
    // then use vector expr to list operands for each destination
    // (we cannot have two repetition list of different lengths in a macro)
    (($vm: expr, $fv: ident) $name: ident:
        BRANCH2 ($($op: ident), *)
            IF (OP $cond: expr)
            THEN $true_dest : ident ($true_args: expr) WITH $prob: expr,
            ELSE $false_dest: ident ($false_args: expr)
    ) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    vec![$($op.clone()),*],
            v:      {
                let true_args = {
                    $true_args.iter().map(|x| DestArg::Normal(*x)).collect()
                };

                let false_args = {
                    $false_args.iter().map(|x| DestArg::Normal(*x)).collect()
                };

                Instruction_::Branch2{
                    cond: $cond,
                    true_dest: Destination {
                        target: $true_dest.hdr.clone(),
                        args: true_args
                    },
                    false_dest: Destination {
                        target: $false_dest.hdr.clone(),
                        args: false_args
                    },
                    true_prob: $prob
                }
            }
        });
    };

    // EXPRCALL
    (($vm: expr, $fv: ident) $name: ident: $res: ident =
     EXPRCALL ($cc: expr, is_abort: $is_abort: expr) $func: ident ($($val: ident), *)) => {
        let ops = vec![$func.clone(), $($val.clone()), *];
        let ops_len = ops.len();
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$res.clone_value()]),
            ops:    ops,
            v:      Instruction_::ExprCall {
                        data: CallData {
                            func: 0,
                            args: (1..ops_len).collect(),
                            convention: $cc
                        },
                        is_abort: $is_abort
                    }
        });
    };
    (($vm: expr, $fv: ident) $name: ident:
     EXPRCALL ($cc: expr, is_abort: $is_abort: expr) $func: ident ($($val: ident), *)) => {
        let ops = vec![$func.clone(), $($val.clone()), *];
        let ops_len = ops.len();
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![]),
            ops:    ops,
            v:      Instruction_::ExprCall {
                        data: CallData {
                            func: 0,
                            args: (1..ops_len).collect(),
                            convention: $cc
                        },
                        is_abort: $is_abort
                    }
        });
    };

    // EXPRCCALL
    (($vm: expr, $fv: ident) $name: ident: $res: ident =
     EXPRCCALL ($cc: expr, is_abort: $is_abort: expr) $func: ident ($($val: ident), *)) => {
        let ops = vec![$func.clone(), $($val.clone()), *];
        let ops_len = ops.len();
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![$res.clone_value()]),
            ops:    ops,
            v:      Instruction_::ExprCCall {
                        data: CallData {
                            func: 0,
                            args: (1..ops_len).collect(),
                            convention: $cc
                        },
                        is_abort: $is_abort
                    }
        });
    };
    (($vm: expr, $fv: ident) $name: ident:
    EXPRCCALL ($cc: expr, is_abort: $is_abort: expr) $func: ident ($($val: ident), *)) => {
        let ops = vec![$func.clone(), $($val.clone()), *];
        let ops_len = ops.len();
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  Some(vec![]),
            ops:    ops,
            v:      Instruction_::ExprCCall {
                        data: CallData {
                            func: 0,
                            args: (1..ops_len).collect(),
                            convention: $cc
                        },
                        is_abort: $is_abort
                    }
        });
    };

    // CALL (1 return result)
    (($vm: expr, $fv: ident) $name: ident: $res: ident =
     CALL ($($op: ident), *) FUNC($func: expr) ($args: expr) $cc: expr,
                      normal: $norm_dest: ident ($norm_args: expr),
                      exc: $exc_dest: ident ($exc_args: expr)) => {
        let $name = $fv.new_inst(Instruction {
            hdr  : MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$res.clone_value()]),
            ops  : vec![$($op.clone()),*],
            v    : Instruction_::Call {
                data: CallData {
                    func: $func,
                    args: $args,
                    convention: $cc
                },
                resume: ResumptionData {
                    normal_dest: Destination {
                        target: $norm_dest.hdr.clone(),
                        args  : $norm_args
                    },
                    exn_dest: Destination {
                        target: $exc_dest.hdr.clone(),
                        args  : $exc_args
                    }
                }
            }
        });
    };
    // CALL (no return value)
    (($vm: expr, $fv: ident) $name: ident:
        CALL ($($op: ident), *) FUNC($func: expr) ($args: expr) $cc: expr,
                      normal: $norm_dest: ident ($norm_args: expr),
                      exc: $exc_dest: ident ($exc_args: expr)) => {
        let $name = $fv.new_inst(Instruction {
            hdr  : MuEntityHeader::unnamed($vm.next_id()),
            value: None,
            ops  : vec![$($op.clone()),*],
            v    : Instruction_::Call {
                data: CallData {
                    func: $func,
                    args: $args,
                    convention: $cc
                },
                resume: ResumptionData {
                    normal_dest: Destination {
                        target: $norm_dest.hdr.clone(),
                        args  : $norm_args
                    },
                    exn_dest: Destination {
                        target: $exc_dest.hdr.clone(),
                        args  : $exc_args
                    }
                }
            }
        });
    };


    // RET
    (($vm: expr, $fv: ident) $name: ident: RET ($($val: ident), +)) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    vec![$($val.clone()), *],
            v:      Instruction_::Return({
                        let mut i = 0;
                        vec![$($val.clone()), *].iter().map(|_| {let ret = i; i+= 1; ret}).collect()
                    })
        });
    };
    // RET (no value)
    (($vm: expr, $fv: ident) $name: ident: RET) => {
        let $name = $fv.new_inst(Instruction{
            hdr:    MuEntityHeader::unnamed($vm.next_id()),
            value:  None,
            ops:    vec![],
            v:      Instruction_::Return(vec![])
        });
    };

    // THREADEXIT
    (($vm: expr, $fv: ident) $name: ident: THREADEXIT) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: None,
            ops: vec![],
            v: Instruction_::ThreadExit
        });
    };

    // THROW
    (($vm: expr, $fv: ident) $name: ident: THROW $op: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: None,
            ops: vec![$op.clone()],
            v: Instruction_::Throw(0)
        });
    };

    // PRINTHEX
    (($vm: expr, $fv: ident) $name: ident: PRINTHEX $val: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: None,
            ops: vec![$val.clone()],
            v: Instruction_::PrintHex(0)
        });
    };

    // SET_RETVAL
    (($vm: expr, $fv: ident) $name: ident: SET_RETVAL $val: ident) => {
        let $name = $fv.new_inst(Instruction{
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: None,
            ops: vec![$val.clone()],
            v: Instruction_::SetRetval(0)
        });
    };

    // MOVE
    (($vm: expr, $fv: ident) $name: ident: MOVE $src: ident -> $dst: ident) => {
        let $name = $fv.new_inst(Instruction {
            hdr: MuEntityHeader::unnamed($vm.next_id()),
            value: Some(vec![$dst.clone_value()]),
            ops: vec![$src],
            v: Instruction_::Move(0)
        });
    };
}

/**************************************
This macro is used as follows:
1- for a test like add_simple(int, int) -> int,
the following syntax should be used (each I  means an int):
      emit_test!      ((vm) (add add_test1 add_test1_v1
        III (sig, int64(22), int64(27), int64(49))));
2- for a test like add_double(double, double) -> double,
the following syntax should be used (each I  means an int):
      emit_test! ((vm) (double_add test1 FFF (sig, f64(1f64), f64(1f64), f64(2f64))));

0- other test types may be manually added using the same approach
***************************************
Macro limitations and points to use:
1 - Macro assumes that the test function signature is named "sig" \
    as currently is.

****************************************/
macro_rules! emit_test {
    /*
    emit_test! ((vm)
        udiv udiv_test1 udiv_test1_v1
        Int,Int, Int
        EQ
        udiv_sig
        int64(22), int64(4), int64(5)
    );
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        $Arg1Type: ident, $Arg2Type: ident RET $Arg3Type: ident,
        $CMPType: ident,
        $test_sig: ident,
        $ty1: ident($in1: expr), $ty2: ident($in2: expr) RET $ty3: ident($out: expr),
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));
        constdef!   (($vm) <$ty1> f64_0 = Constant::$Arg1Type($in1));
        constdef!   (($vm) <$ty2> f64_1 = Constant::$Arg2Type($in2));
        constdef!   (($vm) <$ty3> f64_2 = Constant::$Arg3Type($out));

        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);

        ssa!    (($vm, $tester_name) <$ty1> a);
        ssa!    (($vm, $tester_name) <$ty1> b);

        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));

        // blk_entry
        consta!     (($vm, $tester_name) f64_0_local = f64_0);
        consta!     (($vm, $tester_name) f64_1_local = f64_1);

        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        ssa!    (($vm, $tester_name) <$ty3> result);
        inst!   (($vm, $tester_name) blk_entry_call:
            result = EXPRCALL (CallConvention::Mu, is_abort: false)
                const_funcref_local (f64_0_local, f64_1_local)
        );

        consta!     (($vm, $tester_name) f64_2_local = f64_2);
        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);
        consta!     (($vm, $tester_name) int64_fail_local = int64_fail);
        ssa!    (($vm, $tester_name) <int1> cmp_res);
        inst!   (($vm, $tester_name) blk_entry_cmp:
            cmp_res = CMPOP (CmpOp::$CMPType) result f64_2_local
        );

        ssa!    (($vm, $tester_name) <int32t> blk_entry_ret);
        inst!   (($vm, $tester_name) blk_entry_inst_select:
            blk_entry_ret = SELECT cmp_res int64_pass_local int64_fail_local
        );

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL blk_entry_ret
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );

        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_cmp,
             blk_entry_inst_select,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };
    /*
    emit_test! ((vm) (test_sitofp, test_sitofp_test1, test_sitofp_test1_v1)
        Int -> Double
        EQ
        sig
        int64(-1i64) -> double(-1f64)
    );
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        $Arg1Type: ident RET $Arg3Type: ident,
        $CMPType: ident,
        $test_sig: ident,
        $ty1: ident($in1: expr) RET $ty3: ident($out: expr),
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));
        constdef!   (($vm) <$ty1> f64_0 = Constant::$Arg1Type($in1));
        constdef!   (($vm) <$ty3> f64_2 = Constant::$Arg3Type($out));

        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);
        ssa!    (($vm, $tester_name) <$ty1> a);
        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));
        // blk_entry
        consta!     (($vm, $tester_name) f64_0_local = f64_0);

        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        ssa!    (($vm, $tester_name) <$ty3> result);
        inst!   (($vm, $tester_name) blk_entry_call:
            result = EXPRCALL (CallConvention::Mu, is_abort: false)
                const_funcref_local (f64_0_local)
        );

        consta!     (($vm, $tester_name) f64_2_local = f64_2);
        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);
        consta!     (($vm, $tester_name) int64_fail_local = int64_fail);
        ssa!    (($vm, $tester_name) <int1> cmp_res);
        inst!   (($vm, $tester_name) blk_entry_cmp:
            cmp_res = CMPOP (CmpOp::$CMPType) result f64_2_local
        );

        ssa!    (($vm, $tester_name) <int32t> blk_entry_ret);
        inst!   (($vm, $tester_name) blk_entry_inst_select:
            blk_entry_ret = SELECT cmp_res int64_pass_local int64_fail_local
        );

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL blk_entry_ret
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );
        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_cmp,
             blk_entry_inst_select,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };
    /*
    emit_test!      ((vm) (pass_1arg_by_stack pass_1arg_by_stack_test1 pass_1arg_by_stack_test1_v1
        Int,EQ (sig, int64(1u64))));
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        RET $Arg1Type: ident,
        $CMPType: ident,
        $test_sig: ident,
        RET $ty1: ident($in1: expr),
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));
        constdef!   (($vm) <$ty1> arg_0 = Constant::$Arg1Type($in1));
        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);

        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));

        // blk_entry
        consta!     (($vm, $tester_name) arg_0_local = arg_0);

        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        ssa!    (($vm, $tester_name) <$ty1> result);
        inst!   (($vm, $tester_name) blk_entry_call:
            result = EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_local ()
        );

        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);
        consta!     (($vm, $tester_name) int64_fail_local = int64_fail);
        ssa!    (($vm, $tester_name) <int1> cmp_res);
        inst!   (($vm, $tester_name) blk_entry_cmp:
            cmp_res = CMPOP (CmpOp::$CMPType) result arg_0_local
        );

        ssa!    (($vm, $tester_name) <int32t> blk_entry_ret);
        inst!   (($vm, $tester_name) blk_entry_inst_select:
            blk_entry_ret = SELECT cmp_res int64_pass_local int64_fail_local
        );

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL blk_entry_ret
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );

        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_cmp,
             blk_entry_inst_select,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };

    /*
    emit_test!      ((vm) (catch_exception catch_exception_test1 catch_exception_test1_v1
        (catch_exception_sig)));
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        $test_sig: ident,
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));

        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);

        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));

        // blk_entry
        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        inst!   (($vm, $tester_name) blk_entry_call:
            EXPRCALL (CallConvention::Mu, is_abort: false) const_funcref_local ()
        );

        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL int64_pass_local
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );

        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };

    /*
    emit_test! ((vm) (coalesce_branch2_moves coalesce_branch2_moves_test1
        coalesce_branch2_moves_test1_v1
        Int,Int,Int,Int,Int,Int,Int,EQ
        sig,
        int64(1u64), int64(1u64), int64(10u64),
        int64(10u64), int64(0u64), int64(0u64),
        int64(2u64))));
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        $Arg1Type: ident, $Arg2Type: ident, $Arg3Type: ident,
        $Arg4Type: ident, $Arg5Type: ident, $Arg6Type: ident,
        RET $Arg7Type: ident,
        $CMPType: ident,
        $test_sig: ident,
        $ty1: ident($in1: expr), $ty2: ident($in2: expr), $ty3: ident($in3: expr),
        $ty4: ident($in4: expr), $ty5: ident($in5: expr), $ty6: ident($in6: expr),
        RET $ty7: ident($out: expr),
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));
        constdef!   (($vm) <$ty1> arg_0 = Constant::$Arg1Type($in1));
        constdef!   (($vm) <$ty2> arg_1 = Constant::$Arg2Type($in2));
        constdef!   (($vm) <$ty3> arg_2 = Constant::$Arg3Type($in3));
        constdef!   (($vm) <$ty4> arg_3 = Constant::$Arg4Type($in4));
        constdef!   (($vm) <$ty5> arg_4 = Constant::$Arg5Type($in5));
        constdef!   (($vm) <$ty6> arg_5 = Constant::$Arg6Type($in6));
        constdef!   (($vm) <$ty7> arg_6 = Constant::$Arg7Type($out));

        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);

        ssa!    (($vm, $tester_name) <$ty1> a);
        ssa!    (($vm, $tester_name) <$ty2> b);
        ssa!    (($vm, $tester_name) <$ty3> c);
        ssa!    (($vm, $tester_name) <$ty4> d);
        ssa!    (($vm, $tester_name) <$ty5> e);
        ssa!    (($vm, $tester_name) <$ty6> f);

        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));

        // blk_entry
        consta!     (($vm, $tester_name) arg_0_local = arg_0);
        consta!     (($vm, $tester_name) arg_1_local = arg_1);
        consta!     (($vm, $tester_name) arg_2_local = arg_2);
        consta!     (($vm, $tester_name) arg_3_local = arg_3);
        consta!     (($vm, $tester_name) arg_4_local = arg_4);
        consta!     (($vm, $tester_name) arg_5_local = arg_5);

        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        ssa!    (($vm, $tester_name) <$ty7> result);
        inst!   (($vm, $tester_name) blk_entry_call:
            result = EXPRCALL (CallConvention::Mu, is_abort: false)
                const_funcref_local (arg_0_local, arg_1_local, arg_2_local,
                    arg_3_local, arg_4_local, arg_5_local)
        );

        consta!     (($vm, $tester_name) arg_6_local = arg_6);
        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);
        consta!     (($vm, $tester_name) int64_fail_local = int64_fail);
        ssa!    (($vm, $tester_name) <int1> cmp_res);
        inst!   (($vm, $tester_name) blk_entry_cmp:
            cmp_res = CMPOP (CmpOp::$CMPType) result arg_6_local
        );

        ssa!    (($vm, $tester_name) <int32t> blk_entry_ret);
        inst!   (($vm, $tester_name) blk_entry_inst_select:
            blk_entry_ret = SELECT cmp_res int64_pass_local int64_fail_local
        );

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL blk_entry_ret
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );

        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_cmp,
             blk_entry_inst_select,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };

    /*
    emit_test!      ((vm) (add_twice add_twice_test1 add_twice_test1_v1
        Int,Int,Int,Int,EQ (add_twice_sig, int64(1u64), int64(1u64), int64(1u64), int64(3u64))));
    */
    (($vm: expr)
        $name: ident, $test_name: ident, $tester_name: ident,
        $Arg1Type: ident, $Arg2Type: ident, $Arg3Type: ident,
        RET $Arg4Type: ident,
        $CMPType: ident,
        $test_sig: ident,
        $ty1: ident($in1: expr), $ty2: ident($in2: expr), $ty3: ident($in3: expr),
        RET $ty4: ident($out: expr),
    ) => {
        typedef!    (($vm) int1  = mu_int(1));
        typedef!    (($vm) int32t  = mu_int(32));
        constdef!   (($vm) <int32t> int64_pass = Constant::Int(0));
        constdef!   (($vm) <int32t> int64_fail = Constant::Int(1));
        constdef!   (($vm) <$ty1> arg_0 = Constant::$Arg1Type($in1));
        constdef!   (($vm) <$ty2> arg_1 = Constant::$Arg2Type($in2));
        constdef!   (($vm) <$ty2> arg_2 = Constant::$Arg3Type($in3));
        constdef!   (($vm) <$ty3> arg_3 = Constant::$Arg3Type($out));

        funcsig!    (($vm) tester_sig = () -> ());
        funcdecl!   (($vm) <tester_sig> $test_name);
        funcdef!    (($vm) <tester_sig> $test_name VERSION $tester_name);

        ssa!    (($vm, $tester_name) <$ty1> a);
        ssa!    (($vm, $tester_name) <$ty2> b);
        ssa!    (($vm, $tester_name) <$ty3> c);

        typedef!    (($vm) type_funcref = mu_funcref($test_sig));
        constdef!   (($vm) <type_funcref> const_funcref =
            Constant::FuncRef($name.clone()));

        // blk_entry
        consta!     (($vm, $tester_name) arg_0_local = arg_0);
        consta!     (($vm, $tester_name) arg_1_local = arg_1);
        consta!     (($vm, $tester_name) arg_2_local = arg_2);

        block!      (($vm, $tester_name) blk_entry);

        consta!     (($vm, $tester_name) const_funcref_local = const_funcref);
        ssa!    (($vm, $tester_name) <$ty4> result);
        inst!   (($vm, $tester_name) blk_entry_call:
            result = EXPRCALL (CallConvention::Mu, is_abort: false)
                const_funcref_local (arg_0_local, arg_1_local, arg_2_local)
        );

        consta!     (($vm, $tester_name) arg_3_local = arg_3);
        consta!     (($vm, $tester_name) int64_pass_local = int64_pass);
        consta!     (($vm, $tester_name) int64_fail_local = int64_fail);
        ssa!    (($vm, $tester_name) <int1> cmp_res);
        inst!   (($vm, $tester_name) blk_entry_cmp:
            cmp_res = CMPOP (CmpOp::$CMPType) result arg_3_local
        );

        ssa!    (($vm, $tester_name) <int32t> blk_entry_ret);
        inst!   (($vm, $tester_name) blk_entry_inst_select:
            blk_entry_ret = SELECT cmp_res int64_pass_local int64_fail_local
        );

        inst!   (($vm, $tester_name) blk_entry_inst_ret:
             SET_RETVAL blk_entry_ret
        );
        inst!   (($vm, $tester_name) blk_entry_inst_exit:
            THREADEXIT
        );

        define_block!   (($vm, $tester_name) blk_entry() {
             blk_entry_call,
             blk_entry_cmp,
             blk_entry_inst_select,
             blk_entry_inst_ret,
             blk_entry_inst_exit
        });

        define_func_ver!    (($vm) $tester_name (entry: blk_entry) {
            blk_entry
        });

    };
}

/*
This macro is used as follows:
1 - for add_simple:
    compile_and_run_test! (add, tester_mu);
*/
macro_rules! build_and_run_test {
    ($test_name: ident, $tester_name: ident) => {
        VM::start_logging_trace();

        let vm = Arc::new($test_name());

        let compiler = Compiler::new(CompilerPolicy::default(), &vm);

        let func_id = vm.id_of(stringify!($tester_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        vm.set_primordial_thread(func_id, true, vec![]);

        let func_id = vm.id_of(stringify!($test_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        backend::emit_context(&vm);
        aot::run_test(&vm, stringify!($test_name), stringify!($tester_name));
    };
    // When test name in mu IR is different from the name of rust function \
    // which creates the vm
    ($test_name: ident, $tester_name: ident, $fnc_name: ident) => {
        VM::start_logging_trace();

        let vm = Arc::new($fnc_name());

        let compiler = Compiler::new(CompilerPolicy::default(), &vm);

        let func_id = vm.id_of(stringify!($tester_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        vm.set_primordial_thread(func_id, true, vec![]);

        let func_id = vm.id_of(stringify!($test_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        backend::emit_context(&vm);
        aot::run_test(&vm, stringify!($test_name), stringify!($tester_name));
    };
    // When the testee has one dependent function
    //
    ($test_name: ident AND $dep_name: ident, $tester_name: ident) => {
        VM::start_logging_trace();

        let vm = Arc::new($test_name());

        let compiler = Compiler::new(CompilerPolicy::default(), &vm);

        let func_id = vm.id_of(stringify!($tester_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        vm.set_primordial_thread(func_id, true, vec![]);

        let func_id = vm.id_of(stringify!($test_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }
        let func_id = vm.id_of(stringify!($dep_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        backend::emit_context(&vm);
        aot::run_test_2f(
            &vm,
            stringify!($test_name),
            stringify!($dep_name),
            stringify!($tester_name),
        );
    };
    ($test_name: ident AND $dep_name: ident, $tester_name: ident, $fnc_name: ident) => {
        VM::start_logging_trace();

        let vm = Arc::new($fnc_name());

        let compiler = Compiler::new(CompilerPolicy::default(), &vm);

        let func_id = vm.id_of(stringify!($tester_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        vm.set_primordial_thread(func_id, true, vec![]);

        let func_id = vm.id_of(stringify!($test_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        let func_id = vm.id_of(stringify!($dep_name));
        {
            let funcs = vm.funcs().read().unwrap();
            let func = funcs.get(&func_id).unwrap().read().unwrap();
            let func_vers = vm.func_vers().read().unwrap();
            let mut func_ver = func_vers
                .get(&func.cur_ver.unwrap())
                .unwrap()
                .write()
                .unwrap();

            compiler.compile(&mut func_ver);
        }

        backend::emit_context(&vm);
        aot::run_test_2f(
            &vm,
            stringify!($test_name),
            stringify!($dep_name),
            stringify!($tester_name),
        );
    };
}
