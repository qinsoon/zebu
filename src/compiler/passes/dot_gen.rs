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

#![allow(dead_code)]

use ast::ir::*;
use compiler::CompilerPass;
use std::any::Any;
use vm::VM;

use std::fs::File;
use std::io::prelude::*;
use std::path;
use vm::uir_output::{create_emit_directory, EMIT_MUIR};

fn create_emit_file(name: String, vm: &VM) -> File {
    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push(name);

    match File::create(file_path.as_path()) {
        Err(why) => panic!(
            "couldn't create emit file {}: {}",
            file_path.to_str().unwrap(),
            why
        ),
        Ok(file) => file,
    }
}

pub struct DotGen {
    name: &'static str,
    suffix: &'static str,
}

impl DotGen {
    pub fn new(suffix: &'static str) -> DotGen {
        DotGen {
            name: "DotGen",
            suffix: suffix,
        }
    }
}

#[allow(dead_code)]
fn emit_muir_dot(suffix: &str, func: &MuFunctionVersion, vm: &VM) {
    let func_name = func.name();

    // create emit directory
    create_emit_directory(vm);

    let mut file_path = path::PathBuf::new();
    file_path.push(&vm.vm_options.flag_aot_emit_dir);
    file_path.push((*func_name).clone() + suffix + ".dot");

    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!(
            "couldnt create muir dot {}: {}",
            file_path.to_str().unwrap(),
            why
        ),
        Ok(file) => file,
    };

    emit_muir_dot_inner(&mut file, func_name.clone(), func.content.as_ref().unwrap());
}

fn escape_string(s: String) -> String {
    s.replace("\"", "\\\"") // Replace " with \"
}

fn emit_muir_dot_inner(file: &mut File, f_name: MuName, f_content: &FunctionContent) {
    use utils::vec_utils;

    // digraph func {
    writeln!(file, "digraph {} {{", mangle_name(f_name)).unwrap();

    // node shape: rect
    writeln!(file, "node [shape=rect];").unwrap();

    // every graph node (basic block)
    for (id, block) in f_content.blocks.iter() {
        let block_name = block.hdr.abbreviate_name();
        // BBid [label = "name
        write!(file, "BB{} [label = \"{}", *id, &block_name).unwrap();
        let block_content = block.content.as_ref().unwrap();

        // (args)
        write!(file, "(").unwrap();
        let mut first = true;
        for arg in &block_content.args {
            if !first {
                write!(file, " ").unwrap();
            }
            first = false;
            write!(file, "<{}> {}", arg.ty, arg).unwrap();
        }
        write!(file, ")").unwrap();

        if block_content.exn_arg.is_some() {
            // [exc_arg]
            write!(file, "[{}]", block_content.exn_arg.as_ref().unwrap()).unwrap();
        }

        write!(file, ":\\l").unwrap();

        // all the instructions
        for inst in block_content.body.iter() {
            write!(
                file,
                "    {}\\l",
                escape_string(format!("{}", inst.as_inst()))
            )
            .unwrap();
        }

        // "];
        writeln!(file, "\"];").unwrap();
    }

    // every edge
    for (id, block) in f_content.blocks.iter() {
        use ast::inst::Instruction_::*;

        let cur_block = *id;

        // get last instruction
        let last_inst = block.content.as_ref().unwrap().body.last().unwrap();

        match last_inst.v {
            TreeNode_::Instruction(ref inst) => {
                let ref ops = inst.ops;

                match inst.v {
                    Branch1(ref dest) => {
                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"{}\"];",
                            cur_block,
                            dest.target.id(),
                            vec_utils::as_str(&dest.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    Branch2 {
                        ref true_dest,
                        ref false_dest,
                        ..
                    } => {
                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"true: {}\"]",
                            cur_block,
                            true_dest.target.id(),
                            vec_utils::as_str(&true_dest.get_arguments(&ops))
                        )
                        .unwrap();
                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"false: {}\"]",
                            cur_block,
                            false_dest.target.id(),
                            vec_utils::as_str(&false_dest.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    Switch {
                        ref default,
                        ref branches,
                        ..
                    } => {
                        for &(op, ref dest) in branches.iter() {
                            writeln!(
                                file,
                                "BB{} -> BB{} [label = \"case {}: {}\"]",
                                cur_block,
                                dest.target.id(),
                                ops[op],
                                vec_utils::as_str(&dest.get_arguments(&ops))
                            )
                            .unwrap();
                        }

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"default: {}\"]",
                            cur_block,
                            default.target.id(),
                            vec_utils::as_str(&default.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    Call { ref resume, .. }
                    | CCall { ref resume, .. }
                    | SwapStackExc { ref resume, .. }
                    | ExnInstruction { ref resume, .. } => {
                        let ref normal = resume.normal_dest;
                        let ref exn = resume.exn_dest;

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"normal: {}\"];",
                            cur_block,
                            normal.target.id(),
                            vec_utils::as_str(&normal.get_arguments(&ops))
                        )
                        .unwrap();

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"exception: {}\"];",
                            cur_block,
                            exn.target.id(),
                            vec_utils::as_str(&exn.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    Watchpoint {
                        ref id,
                        ref disable_dest,
                        ref resume,
                        ..
                    } if id.is_some() => {
                        let ref normal = resume.normal_dest;
                        let ref exn = resume.exn_dest;

                        if id.is_some() {
                            let disable_dest = disable_dest.as_ref().unwrap();
                            writeln!(
                                file,
                                "BB{} -> {} [label = \"disabled: {}\"];",
                                cur_block,
                                disable_dest.target.id(),
                                vec_utils::as_str(&disable_dest.get_arguments(&ops))
                            )
                            .unwrap();
                        }

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"normal: {}\"];",
                            cur_block,
                            normal.target.id(),
                            vec_utils::as_str(&normal.get_arguments(&ops))
                        )
                        .unwrap();

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"exception: {}\"];",
                            cur_block,
                            exn.target.id(),
                            vec_utils::as_str(&exn.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    WPBranch {
                        ref disable_dest,
                        ref enable_dest,
                        ..
                    } => {
                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"disabled: {}\"];",
                            cur_block,
                            disable_dest.target.id(),
                            vec_utils::as_str(&disable_dest.get_arguments(&ops))
                        )
                        .unwrap();

                        writeln!(
                            file,
                            "BB{} -> BB{} [label = \"enabled: {}\"];",
                            cur_block,
                            enable_dest.target.id(),
                            vec_utils::as_str(&enable_dest.get_arguments(&ops))
                        )
                        .unwrap();
                    }
                    Return(_) | Throw(_) | ThreadExit | TailCall(_) | SwapStackKill { .. } => {}

                    _ => {
                        panic!("unexpected terminating instruction: {}", inst);
                    }
                }
            }
            TreeNode_::Value(_) => {
                panic!("expected last tree node to be an instruction");
            }
        }
    }

    writeln!(file, "}}").unwrap();
}

impl CompilerPass for DotGen {
    fn name(&self) -> &'static str {
        self.name
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn visit_function(&mut self, vm: &VM, func: &mut MuFunctionVersion) {
        if EMIT_MUIR {
            emit_muir_dot(self.suffix, func, vm);
        }
    }
}
