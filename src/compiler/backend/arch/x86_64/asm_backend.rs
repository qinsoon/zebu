#![allow(unused_variables)]

use compiler::backend;
use compiler::backend::AOT_EMIT_CONTEXT_FILE;
use compiler::backend::AOT_EMIT_DIR;
use compiler::backend::RegGroup;
use utils::ByteSize;
use compiler::backend::x86_64;
use compiler::backend::x86_64::CodeGenerator;
use compiler::machine_code::MachineCode;
use vm::VM;
use runtime::ValueLocation;

use utils::vec_utils;
use utils::string_utils;
use ast::ptr::P;
use ast::ir::*;

use std::collections::HashMap;
use std::str;
use std::usize;
use std::slice::Iter;
use std::ops;

struct ASMCode {
    name: MuName, 
    code: Vec<ASMInst>,
    
    idx_to_blk: HashMap<usize, MuName>,
    blk_to_idx: HashMap<MuName, usize>,
    
    blocks: Vec<MuName>,
    block_start: HashMap<MuName, usize>,
    block_range: HashMap<MuName, ops::Range<usize>>,
    
    block_livein: HashMap<MuName, Vec<MuID>>,
    block_liveout: HashMap<MuName, Vec<MuID>>
}

unsafe impl Send for ASMCode {} 
unsafe impl Sync for ASMCode {}

impl ASMCode {
    fn get_use_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.uses.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                },
                None => {}
            }
        }

        ret
    }

    fn get_define_locations(&self, reg: MuID) -> Vec<ASMLocation> {
        let mut ret = vec![];

        for inst in self.code.iter() {
            match inst.defines.get(&reg) {
                Some(ref locs) => {
                    ret.append(&mut locs.to_vec());
                },
                None => {}
            }
        }

        ret
    }

    fn rewrite_insert(
        &self,
        insert_before: HashMap<usize, Vec<Box<ASMCode>>>,
        insert_after: HashMap<usize, Vec<Box<ASMCode>>>) -> Box<ASMCode>
    {
        let mut ret = ASMCode {
            name: self.name.clone(),
            code: vec![],
            idx_to_blk: HashMap::new(),
            blk_to_idx: HashMap::new(),
            blocks: vec![],
            block_start: HashMap::new(),
            block_range: HashMap::new(),
            block_livein: HashMap::new(),
            block_liveout: HashMap::new()
        };

        // iterate through old machine code
        let mut inst_offset = 0;    // how many instructions has been inserted

        for i in 0..self.number_of_insts() {
            // insert code before this instruction
            if insert_before.contains_key(&i) {
                for insert in insert_before.get(&i).unwrap() {
                    ret.append_code_sequence_all(insert);
                    inst_offset += insert.number_of_insts();
                }
            }

            // copy this instruction


            // insert code after this instruction
            if insert_after.contains_key(&i) {

            }
        }

        unimplemented!()
    }

    fn append_code_sequence(
        &mut self,
        another: &Box<ASMCode>,
        start_inst: usize,
        n_insts: usize)
    {
        let self_index = self.number_of_insts();
        unimplemented!()
    }

    fn append_code_sequence_all(&mut self, another: &Box<ASMCode>) {
        let n_insts = another.number_of_insts();
        self.append_code_sequence(another, 0, n_insts)
    }
}

use std::any::Any;

impl MachineCode for ASMCode {
    fn as_any(&self) -> &Any {
        self
    }
    fn number_of_insts(&self) -> usize {
        self.code.len()
    }
    
    fn is_move(&self, index: usize) -> bool {
        let inst = self.code.get(index);
        match inst {
            Some(inst) => inst.code.starts_with("mov"),
            None => false
        }
    }
    
    fn is_using_mem_op(&self, index: usize) -> bool {
        self.code[index].is_mem_op_used
    }
    
    fn get_succs(&self, index: usize) -> &Vec<usize> {
        &self.code[index].succs
    }
    
    fn get_preds(&self, index: usize) -> &Vec<usize> {
        &self.code[index].preds
    }
    
    fn get_inst_reg_uses(&self, index: usize) -> Vec<MuID> {
        self.code[index].uses.keys().map(|x| *x).collect()
    }
    
    fn get_inst_reg_defines(&self, index: usize) -> Vec<MuID> {
        self.code[index].defines.keys().map(|x| *x).collect()
    }
    
    fn replace_reg(&mut self, from: MuID, to: MuID) {
        let to_reg_tag : MuName = match backend::all_regs().get(&to) {
            Some(reg) => reg.name().unwrap(),
            None => panic!("expecting a machine register, but we are required to replace to {}", to)
        };
        let to_reg_string = "%".to_string() + &to_reg_tag;

        for loc in self.get_define_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];
            for i in 0..loc.len {
                // FIXME: why loop here?
                string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
            }
        }

        for loc in self.get_use_locations(from) {
            let ref mut inst_to_patch = self.code[loc.line];
            for i in 0..loc.len {
                string_utils::replace(&mut inst_to_patch.code, loc.index, &to_reg_string, to_reg_string.len());
            }
        }
    }

    fn replace_tmp_for_inst(&mut self, from: MuID, to: MuID, inst: usize) {
        let to_reg_string : MuName = match backend::all_regs().get(&to) {
            Some(ref machine_reg) => {
                let name = machine_reg.name().unwrap();
                "%".to_string() + &name
            },
            None => REG_PLACEHOLDER.clone()
        };

        let asm = &mut self.code[inst];
        // if this reg is defined, replace the define
        if asm.defines.contains_key(&from) {
            let define_locs = asm.defines.get(&from).unwrap().to_vec();
            // replace temps
            for loc in define_locs.iter() {
                for i in 0..loc.len {
                    string_utils::replace(&mut asm.code, loc.index, &to_reg_string, to_reg_string.len());
                }
            }

            // remove old key, insert new one
            asm.defines.remove(&from);
            asm.defines.insert(from, define_locs);
        }

        // if this reg is used, replace the use
        if asm.uses.contains_key(&from) {
            let use_locs = asm.uses.get(&from).unwrap().to_vec();
            // replace temps
            for loc in use_locs.iter() {
                for i in 0..loc.len {
                    string_utils::replace(&mut asm.code, loc.index, &to_reg_string, to_reg_string.len());
                }
            }

            // remove old key, insert new one
            asm.uses.remove(&from);
            asm.uses.insert(from, use_locs);
        }
    }
    
    fn set_inst_nop(&mut self, index: usize) {
        self.code.remove(index);
        self.code.insert(index, ASMInst::nop());
    }
    
    fn emit(&self) -> Vec<u8> {
        let mut ret = vec![];
        
        for inst in self.code.iter() {
            ret.append(&mut inst.code.clone().into_bytes());
            ret.append(&mut "\n".to_string().into_bytes());
        }
        
        ret
    }
    
    fn trace_mc(&self) {
        trace!("");

        trace!("code for {}: \n", self.name);
        
        let n_insts = self.code.len();
        for i in 0..n_insts {
            self.trace_inst(i);
        }
        
        trace!("")      
    }
    
    fn trace_inst(&self, i: usize) {
        trace!("#{}\t{:30}\t\tdefine: {:?}\tuses: {:?}\tpred: {:?}\tsucc: {:?}", 
            i, self.code[i].code, self.get_inst_reg_defines(i), self.get_inst_reg_uses(i),
            self.code[i].preds, self.code[i].succs);
    }
    
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>> {
        self.block_livein.get(&block.to_string())
    }
    
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>> {
        self.block_liveout.get(&block.to_string())
    }
    
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>) {
        self.block_livein.insert(block.to_string(), set);
    }
    
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>) {
        self.block_liveout.insert(block.to_string(), set);
    }
    
    fn get_all_blocks(&self) -> &Vec<MuName> {
        &self.blocks
    }
    
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>> {
        match self.block_range.get(&block.to_string()) {
            Some(r) => Some(r.clone()),
            None => None
        }
    }
}

#[derive(Clone, Debug)]
enum ASMBranchTarget {
    None,
    Conditional(MuName),
    Unconditional(MuName)
}

struct ASMInst {
    code: String,

    defines: HashMap<MuID, Vec<ASMLocation>>,
    uses: HashMap<MuID, Vec<ASMLocation>>,

    is_mem_op_used: bool,
    preds: Vec<usize>,
    succs: Vec<usize>,
    branch: ASMBranchTarget
}

impl ASMInst {
    fn symbolic(line: String) -> ASMInst {
        ASMInst {
            code: line,
            defines: HashMap::new(),
            uses: HashMap::new(),
            is_mem_op_used: false,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None
        }
    }
    
    fn inst(
        inst: String,
        defines: HashMap<MuID, Vec<ASMLocation>>,
        uses: HashMap<MuID, Vec<ASMLocation>>,
        is_mem_op_used: bool,
        target: ASMBranchTarget
    ) -> ASMInst
    {
        ASMInst {
            code: inst,
            defines: defines,
            uses: uses,
            is_mem_op_used: is_mem_op_used,
            preds: vec![],
            succs: vec![],
            branch: target
        }
    }
    
    fn nop() -> ASMInst {
        ASMInst {
            code: "".to_string(),
            defines: HashMap::new(),
            uses: HashMap::new(),
            is_mem_op_used: false,
            preds: vec![],
            succs: vec![],
            branch: ASMBranchTarget::None
        }
    }
}

#[derive(Clone, Debug)]
struct ASMLocation {
    line: usize,
    index: usize,
    len: usize
}

impl ASMLocation {
    fn new(line: usize, index: usize, len: usize) -> ASMLocation {
        ASMLocation{
            line: line,
            index: index,
            len: len
        }
    }
}

pub struct ASMCodeGen {
    cur: Option<Box<ASMCode>>
}

const REG_PLACEHOLDER_LEN : usize = 5;
lazy_static! {
    pub static ref REG_PLACEHOLDER : String = {
        let blank_spaces = [' ' as u8; REG_PLACEHOLDER_LEN];
        
        format!("%{}", str::from_utf8(&blank_spaces).unwrap())
    };
}

impl ASMCodeGen {
    pub fn new() -> ASMCodeGen {
        ASMCodeGen {
            cur: None
        }
    }
    
    fn cur(&self) -> &ASMCode {
        self.cur.as_ref().unwrap()
    }
    
    fn cur_mut(&mut self) -> &mut ASMCode {
        self.cur.as_mut().unwrap()
    }
    
    fn line(&self) -> usize {
        self.cur().code.len()
    }
    
    fn add_asm_label(&mut self, code: String) {
        let l = self.line();
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }
    
    fn add_asm_block_label(&mut self, code: String, block_name: MuName) {
        let l = self.line();
        self.cur_mut().code.push(ASMInst::symbolic(code));
        
        self.cur_mut().idx_to_blk.insert(l, block_name.clone());
        self.cur_mut().blk_to_idx.insert(block_name, l);
    }
    
    fn add_asm_symbolic(&mut self, code: String){
        self.cur_mut().code.push(ASMInst::symbolic(code));
    }
    
    fn prepare_machine_regs(&self, regs: Iter<P<Value>>) -> Vec<MuID> {
        regs.map(|x| self.prepare_machine_reg(x)).collect()
    }
    
    fn add_asm_call(&mut self, code: String) {
        // a call instruction will use all the argument registers
        let mut uses : HashMap<MuID, Vec<ASMLocation>> = HashMap::new();
        for reg in x86_64::ARGUMENT_GPRs.iter() {
            uses.insert(reg.id(), vec![]);
        }
        for reg in x86_64::ARGUMENT_FPRs.iter() {
            uses.insert(reg.id(), vec![]);
        }

        // defines: return registers
        let mut defines : HashMap<MuID, Vec<ASMLocation>> = HashMap::new();
        for reg in x86_64::RETURN_GPRs.iter() {
            defines.insert(reg.id(), vec![]);
        }
        for reg in x86_64::RETURN_FPRs.iter() {
            defines.insert(reg.id(), vec![]);
        }
        for reg in x86_64::CALLER_SAVED_GPRs.iter() {
            if !defines.contains_key(&reg.id()) {
                defines.insert(reg.id(), vec![]);
            }
        }
        for reg in x86_64::CALLER_SAVED_FPRs.iter() {
            if !defines.contains_key(&reg.id()) {
                defines.insert(reg.id(), vec![]);
            }
        }
          
        self.add_asm_inst(code, defines, uses, false);
    }
    
    fn add_asm_ret(&mut self, code: String) {
        let mut uses : HashMap<MuID, Vec<ASMLocation>> = {
            let mut ret = HashMap::new();
            for reg in x86_64::RETURN_GPRs.iter() {
                ret.insert(reg.id(), vec![]);
            }
            for reg in x86_64::RETURN_FPRs.iter() {
                ret.insert(reg.id(), vec![]);
            }
            ret
        };
        
        self.add_asm_inst(code, hashmap!{}, uses, false);
    }
    
    fn add_asm_branch(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(code, hashmap!{}, hashmap!{}, false, ASMBranchTarget::Unconditional(target));
    }
    
    fn add_asm_branch2(&mut self, code: String, target: MuName) {
        self.add_asm_inst_internal(code, hashmap!{}, hashmap!{}, false, ASMBranchTarget::Conditional(target));
    }
    
    fn add_asm_inst(
        &mut self, 
        code: String, 
        defines: HashMap<MuID, Vec<ASMLocation>>,
        uses: HashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool)
    {
        self.add_asm_inst_internal(code, defines, uses, is_using_mem_op, ASMBranchTarget::None)
    }

    fn add_asm_inst_internal(
        &mut self,
        code: String,
        defines: HashMap<MuID, Vec<ASMLocation>>,
        uses: HashMap<MuID, Vec<ASMLocation>>,
        is_using_mem_op: bool,
        target: ASMBranchTarget)
    {
        let line = self.line();
        trace!("asm: {}", code);
        trace!("     defines: {:?}", defines);
        trace!("     uses: {:?}", uses);
        let mc = self.cur_mut();

        // put the instruction
        mc.code.push(ASMInst::inst(code, defines, uses, is_using_mem_op, target));
    }
    
    fn prepare_reg(&self, op: &P<Value>, loc: usize) -> (String, MuID, ASMLocation) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {},
                _ => panic!("expecting register op")
            }
        }
        
        let str = self.asm_reg_op(op);
        let len = str.len();
        (str, op.extract_ssa_id().unwrap(), ASMLocation::new(self.line(), loc, len))
    }
    
    fn prepare_machine_reg(&self, op: &P<Value>) -> MuID {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::SSAVar(_) => {},
                _ => panic!("expecting machine register op")
            }
        }        
        
        op.extract_ssa_id().unwrap()
    }
    
    #[allow(unused_assignments)]
    fn prepare_mem(&self, op: &P<Value>, loc: usize) -> (String, HashMap<MuID, Vec<ASMLocation>>) {
        if cfg!(debug_assertions) {
            match op.v {
                Value_::Memory(_) => {},
                _ => panic!("expecting register op")
            }
        }        

        let mut ids : Vec<MuID> = vec![];
        let mut locs : Vec<ASMLocation> = vec![];
        let mut result_str : String = "".to_string();
        
        let mut loc_cursor : usize = loc;
        
        match op.v {
            // offset(base,index,scale)
            Value_::Memory(MemoryLocation::Address{ref base, ref offset, ref index, scale}) => {
                // deal with offset
                if offset.is_some() {
                    let offset = offset.as_ref().unwrap();
                    
                    match offset.v {
                        Value_::SSAVar(id) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(offset, 0);
                            
                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            
                            loc_cursor += str.len();
                        },
                        Value_::Constant(Constant::Int(val)) => {
                            let str = val.to_string();
                            
                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        },
                        _ => panic!("unexpected offset type: {:?}", offset)
                    }
                }
                
                result_str.push('(');
                loc_cursor += 1; 
                
                // deal with base, base is ssa
                let (str, id, loc) = self.prepare_reg(base, loc_cursor);
                result_str.push_str(&str);
                ids.push(id);
                locs.push(loc);
                loc_cursor += str.len();
                
                // deal with index (ssa or constant)
                if index.is_some() {
                    result_str.push(',');
                    loc_cursor += 1; // plus 1 for ,                    
                    
                    let index = index.as_ref().unwrap();
                    
                    match index.v {
                        Value_::SSAVar(id) => {
                            // temp as offset
                            let (str, id, loc) = self.prepare_reg(index, loc_cursor);
                            
                            result_str.push_str(&str);
                            ids.push(id);
                            locs.push(loc);
                            
                            loc_cursor += str.len();
                        },
                        Value_::Constant(Constant::Int(val)) => {
                            let str = val.to_string();
                            
                            result_str.push_str(&str);
                            loc_cursor += str.len();
                        },
                        _ => panic!("unexpected index type: {:?}", index)
                    }
                    
                    // scale
                    if scale.is_some() {
                        result_str.push(',');
                        loc_cursor += 1;
                        
                        let scale = scale.unwrap();
                        let str = scale.to_string();
                        
                        result_str.push_str(&str);
                        loc_cursor += str.len();
                    }
                }
                
                result_str.push(')');
                loc_cursor += 1;
            },
            Value_::Memory(MemoryLocation::Symbolic{ref base, ref label}) => {
                result_str.push_str(&symbol(label.clone()));
                loc_cursor += label.len();
                
                if base.is_some() {
                    result_str.push('(');
                    loc_cursor += 1;
                    
                    let (str, id, loc) = self.prepare_reg(base.as_ref().unwrap(), loc_cursor);
                    result_str.push_str(&str);
                    ids.push(id);
                    locs.push(loc);
                    loc_cursor += str.len();

                    result_str.push(')');
                    loc_cursor += 1;
                }
            },
            _ => panic!("expect mem location as value")
        }

        let uses : HashMap<MuID, Vec<ASMLocation>> = {
            let mut map : HashMap<MuID, Vec<ASMLocation>> = hashmap!{};
            for i in 0..ids.len() {
                let id = ids[i];
                let loc = locs[i].clone();

                if map.contains_key(&id) {
                    map.get_mut(&id).unwrap().push(loc);
                } else {
                    map.insert(id, vec![loc]);
                }
            }
            map
        };


        (result_str, uses)
    }
    
    fn asm_reg_op(&self, op: &P<Value>) -> String {
        let id = op.extract_ssa_id().unwrap();
        if id < MACHINE_ID_END {
            // machine reg
            format!("%{}", op.name().unwrap())
        } else {
            // virtual register, use place holder
            REG_PLACEHOLDER.clone()
        }
    }
    
    fn asm_block_label(&self, label: MuName) -> String {
        symbol(format!("{}_{}", self.cur().name, label))
    }
    
    fn control_flow_analysis(&mut self) {
        // control flow analysis
        let n_insts = self.line();

        let code = self.cur_mut();
        let ref mut asm = code.code;
        
        for i in 0..n_insts {
            // determine predecessor - if cur is not block start, its predecessor is previous insts
            let is_block_start = code.idx_to_blk.get(&i);
            if is_block_start.is_none() {
                if i > 0 {
                    trace!("inst {}: not a block start", i);
                    trace!("inst {}: set PREDS as previous inst {}", i, i-1);
                    asm[i].preds.push(i - 1);
                }
            } else {
                // if cur is a branch target, we already set its predecessor
                // if cur is a fall-through block, we set it in a sanity check pass
            }
            
            // determine successor
            let branch = asm[i].branch.clone();
            match branch {
                ASMBranchTarget::Unconditional(ref target) => {
                    // branch to target
                    trace!("inst {}: is a branch to {}", i, target);

                    let target_n = code.blk_to_idx.get(target).unwrap();
                    trace!("inst {}: branch target index is {}", i, target_n);

                    // cur inst's succ is target
                    trace!("inst {}: set SUCCS as branch target {}", i, target_n);
                    asm[i].succs.push(*target_n);

                    // target's pred is cur
                    trace!("inst {}: set PREDS as branch source {}", target_n, i);
                    asm[*target_n].preds.push(i);
                },
                ASMBranchTarget::Conditional(ref target) => {
                    // branch to target
                    trace!("inst {}: is a cond branch to {}", i, target);

                    let target_n = code.blk_to_idx.get(target).unwrap();
                    trace!("inst {}: branch target index is {}", i, target_n);

                    // cur insts' succ is target and next inst
                    asm[i].succs.push(*target_n);
                    trace!("inst {}: set SUCCS as branch target {}", i, target_n);
                    if i < n_insts - 1 {
                        trace!("inst {}: set SUCCS as next inst", i + 1);
                        asm[i].succs.push(i + 1);
                    }

                    // target's pred is cur
                    asm[*target_n].preds.push(i);
                    trace!("inst {}: set PREDS as {}", *target_n, i);
                },
                ASMBranchTarget::None => {
                    // not branch nor cond branch, succ is next inst
                    trace!("inst {}: not a branch inst", i);
                    if i < n_insts - 1 {
                        trace!("inst {}: set SUCCS as next inst {}", i, i + 1);
                        asm[i].succs.push(i + 1);
                    }
                }
            }
        }
        
        // a sanity check for fallthrough blocks
        for i in 0..n_insts {
            if i != 0 && asm[i].preds.len() == 0 {
                asm[i].preds.push(i - 1);
            }
        }        
    }

    fn finish_code_sequence_asm(&mut self) -> Box<ASMCode> {
        self.cur.take().unwrap()
    }
}

impl CodeGenerator for ASMCodeGen {
    fn start_code(&mut self, func_name: MuName) -> ValueLocation {
        self.cur = Some(Box::new(ASMCode {
                name: func_name.clone(),
                code: vec![],
                
                idx_to_blk: HashMap::new(),
                blk_to_idx: HashMap::new(),
                
                blocks: vec![],
                block_start: HashMap::new(),
                block_range: HashMap::new(),
                
                block_livein: HashMap::new(),
                block_liveout: HashMap::new()
            }));
        
        // to link with C sources via gcc
        let func_symbol = symbol(func_name.clone());
        self.add_asm_symbolic(directive_globl(func_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", func_symbol.clone()));
        
        ValueLocation::Relocatable(RegGroup::GPR, func_name)
    }
    
    fn finish_code(&mut self, func_name: MuName) -> (Box<MachineCode + Sync + Send>, ValueLocation) {
        let func_end = {
            let mut symbol = func_name.clone();
            symbol.push_str("_end");
            symbol
        };
        self.add_asm_symbolic(directive_globl(symbol(func_end.clone())));
        self.add_asm_symbolic(format!("{}:", symbol(func_end.clone())));
        
        self.control_flow_analysis();
        
        (
            self.cur.take().unwrap(),
            ValueLocation::Relocatable(RegGroup::GPR, func_end)
        )
    }

    fn start_code_sequence(&mut self) {
        self.cur = Some(Box::new(ASMCode {
            name: "snippet".to_string(),
            code: vec![],

            idx_to_blk: HashMap::new(),
            blk_to_idx: HashMap::new(),

            blocks: vec![],
            block_start: HashMap::new(),
            block_range: HashMap::new(),

            block_livein: HashMap::new(),
            block_liveout: HashMap::new()
        }));
    }

    fn finish_code_sequence(&mut self) -> Box<MachineCode + Sync + Send> {
        self.finish_code_sequence_asm()
    }

    fn print_cur_code(&self) {
        println!("");
        
        if self.cur.is_some() {
            let code = self.cur.as_ref().unwrap();
            
            println!("code for {}: ", code.name);
            let n_insts = code.code.len();
            for i in 0..n_insts {
                let ref line = code.code[i];
                println!("#{}\t{}", i, line.code);
            }
        } else {
            println!("no current code");
        }
        
        println!("");
    }
    
    fn start_block(&mut self, block_name: MuName) {
        let label = format!("{}:", self.asm_block_label(block_name.clone()));        
        self.add_asm_block_label(label, block_name.clone());
        self.cur_mut().blocks.push(block_name.clone());
        
        let start = self.line();
        self.cur_mut().block_start.insert(block_name, start);
    }
    
    fn start_exception_block(&mut self, block_name: MuName) -> ValueLocation {
        let block = self.asm_block_label(block_name.clone());
        self.add_asm_symbolic(directive_globl(symbol(block.clone())));
        self.add_asm_symbolic(format!("{}:", symbol(block.clone())));
        
        self.start_block(block_name);
        
        ValueLocation::Relocatable(RegGroup::GPR, block)
    }
    
    fn end_block(&mut self, block_name: MuName) {
        let start : usize = *self.cur().block_start.get(&block_name).unwrap();
        let end : usize = self.line();
        
        self.cur_mut().block_range.insert(block_name, (start..end));
    }
    
    fn set_block_livein(&mut self, block_name: MuName, live_in: &Vec<P<Value>>) {
        let cur = self.cur_mut();
        
        let mut res = {
            if !cur.block_livein.contains_key(&block_name) {
                cur.block_livein.insert(block_name.clone(), vec![]);
            } else {
                panic!("seems we are inserting livein to block {} twice", block_name);
            }
            
            cur.block_livein.get_mut(&block_name).unwrap()
        };
        
        for value in live_in {
            res.push(value.extract_ssa_id().unwrap());
        }
    }
    
    fn set_block_liveout(&mut self, block_name: MuName, live_out: &Vec<P<Value>>) {
        let cur = self.cur_mut();
        
        let mut res = {
            if !cur.block_liveout.contains_key(&block_name) {
                cur.block_liveout.insert(block_name.clone(), vec![]);
            } else {
                panic!("seems we are inserting livein to block {} twice", block_name);
            }
            
            cur.block_liveout.get_mut(&block_name).unwrap()
        };
        
        for value in live_out {
            match value.extract_ssa_id() {
                Some(id) => res.push(id),
                None => {}
            }
        }        
    }
    
    fn emit_nop(&mut self, bytes: usize) {
        trace!("emit: nop ({} bytes)", bytes);
        
        let asm = String::from("nop");
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            hashmap!{},
            false
        );
    }
    
    fn emit_cmp_r64_r64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_reg(op1, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(op2, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("cmpq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            hashmap!{
                id1 => vec![loc1],
                id2 => vec![loc2]
            },
            false
        );
    }
    
    fn emit_cmp_r64_imm32(&mut self, op1: &P<Value>, op2: i32) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg1, id1, loc1) = self.prepare_reg(op1, 4 + 1 + 1 + op2.to_string().len() + 1);
        
        let asm = format!("cmpq ${},{}", op2, reg1);
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }
    
    fn emit_cmp_r64_mem64(&mut self, op1: &P<Value>, op2: &P<Value>) {
        trace!("emit: cmp {} {}", op1, op2);
        
        let (reg, id1, loc1) = self.prepare_reg(op1, 4 + 1);
        let (mem, mut uses) = self.prepare_mem(op2, 4 + 1 + reg.len() + 1);
        
        let asm = format!("cmpq {},{}", reg, mem);
        
        // merge use vec
        if uses.contains_key(&id1) {
            uses.get_mut(&id1).unwrap().push(loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            uses,
            true
        )
    }
    
    fn emit_mov_r64_imm32(&mut self, dest: &P<Value>, src: i32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("movq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id1 => vec![loc1]
            },
            hashmap!{},
            false
        )
    }
    
    // load
    fn emit_mov_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (mem, uses) = self.prepare_mem(src, 4 + 1);
        let (reg, id2, loc2) = self.prepare_reg(dest, 4 + 1 + mem.len() + 1);
        
        let asm = format!("movq {},{}", mem, reg);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        )
    }
    
    // store
    fn emit_mov_mem64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (mem, mut uses) = self.prepare_mem(dest, 4 + 1 + reg.len() + 1);
        
        // the register we used for the memory location is counted as 'use'
        // use the vec from mem as 'use' (push use reg from src to it)
        if uses.contains_key(&id1) {
            uses.get_mut(&id1).unwrap().push(loc1);
        } else {
            uses.insert(id1, vec![loc1]);
        }
        
        let asm = format!("movq {},{}", reg, mem);
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            uses,
            true
        )
    }
    
    fn emit_mov_mem64_imm32(&mut self, dest: &P<Value>, src: i32) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (mem, uses) = self.prepare_mem(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("movq ${},{}", src, mem);
        
        self.add_asm_inst(
            asm,
            hashmap!{},
            uses,
            true
        )
    }
    
    fn emit_mov_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: mov {} -> {}", src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("movq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2]
            },
            hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }
    
    fn emit_add_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("addq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2.clone()]
            },
            hashmap!{
                id1 => vec![loc1],
                id2 => vec![loc2]
            },
            false
        )
    }
    
    fn emit_lea_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: lea {} -> {}", src, dest);
        
        let (mem, uses) = self.prepare_mem(src, 4 + 1);
        let (reg, id2, loc2) = self.prepare_reg(dest, 4 + 1 + mem.len() + 1);
        
        let asm = format!("leaq {},{}", mem, reg);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2]
            },
            uses,
            true
        ) 
    }
    
    fn emit_and_r64_imm32(&mut self, dest: &P<Value>, src: i32) {
        trace!("emit: and {}, {} -> {}", src, dest, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);

        let asm = format!("andq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id1 => vec![loc1.clone()]
            },
            hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }
    
    fn emit_and_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: and {}, {} -> {}", src, dest, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1); 

        let asm = format!("andq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2.clone()]
            },
            hashmap!{
                id1 => vec![loc1],
                id2 => vec![loc2]
            },
            false
        )
    }    
    
    fn emit_add_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_add_r64_imm32(&mut self, dest: &P<Value>, src: i32) {
        trace!("emit: add {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("addq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id1 => vec![loc1.clone()]
            },
            hashmap!{
                id1 => vec![loc1]
            },
            false
        )
    }
    
    fn emit_sub_r64_r64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(src, 4 + 1);
        let (reg2, id2, loc2) = self.prepare_reg(dest, 4 + 1 + reg1.len() + 1);
        
        let asm = format!("subq {},{}", reg1, reg2);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id2 => vec![loc2.clone()]
            },
            hashmap!{
                id1 => vec![loc1],
                id2 => vec![loc2]
            },
            false
        )        
    }
    
    fn emit_sub_r64_mem64(&mut self, dest: &P<Value>, src: &P<Value>) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        unimplemented!()
    }
    
    fn emit_sub_r64_imm32(&mut self, dest: &P<Value>, src: i32) {
        trace!("emit: sub {}, {} -> {}", dest, src, dest);
        
        let (reg1, id1, loc1) = self.prepare_reg(dest, 4 + 1 + 1 + src.to_string().len() + 1);
        
        let asm = format!("subq ${},{}", src, reg1);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id1 => vec![loc1.clone()]
            },
            hashmap!{
                id1 => vec![loc1]
            },
            false
        )        
    }
    
    fn emit_mul_r64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> (rdx, rax)", src);
        
        let (reg, id, loc) = self.prepare_reg(src, 3 + 1);
        let rax = self.prepare_machine_reg(&x86_64::RAX);
        let rdx = self.prepare_machine_reg(&x86_64::RDX);
        
        let asm = format!("mul {}", reg);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                rax => vec![],
                rdx => vec![]
            },
            hashmap!{
                id => vec![loc],
                rax => vec![]
            },
            false
        )
    }
    
    fn emit_mul_mem64(&mut self, src: &P<Value>) {
        trace!("emit: mul rax, {} -> rax", src);
        unimplemented!()
    }
    
    fn emit_jmp(&mut self, dest_name: MuName) {
        trace!("emit: jmp {}", dest_name);
        
        // symbolic label, we dont need to patch it
        let asm = format!("jmp {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch(asm, dest_name)
    }
    
    fn emit_je(&mut self, dest_name: MuName) {
        trace!("emit: je {}", dest_name);
        
        let asm = format!("je {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jne(&mut self, dest_name: MuName) {
        trace!("emit: jne {}", dest_name);
        
        let asm = format!("jne {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_ja(&mut self, dest_name: MuName) {
        trace!("emit: ja {}", dest_name);
        
        let asm = format!("ja {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jae(&mut self, dest_name: MuName) {
        trace!("emit: jae {}", dest_name);
        
        let asm = format!("jae {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jb(&mut self, dest_name: MuName) {
        trace!("emit: jb {}", dest_name);
        
        let asm = format!("jb {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);
    }
    
    fn emit_jbe(&mut self, dest_name: MuName) {
        trace!("emit: jbe {}", dest_name);
        
        let asm = format!("jbe {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jg(&mut self, dest_name: MuName) {
        trace!("emit: jg {}", dest_name);
        
        let asm = format!("jg {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jge(&mut self, dest_name: MuName) {
        trace!("emit: jge {}", dest_name);
        
        let asm = format!("jge {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jl(&mut self, dest_name: MuName) {
        trace!("emit: jl {}", dest_name);
        
        let asm = format!("jl {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }
    
    fn emit_jle(&mut self, dest_name: MuName) {
        trace!("emit: jle {}", dest_name);
        
        let asm = format!("jle {}", self.asm_block_label(dest_name.clone()));
        self.add_asm_branch2(asm, dest_name);        
    }    
    
    fn emit_call_near_rel32(&mut self, callsite: String, func: MuName) -> ValueLocation {
        trace!("emit: call {}", func);
        
        let asm = format!("call {}", symbol(func));
        self.add_asm_call(asm);
        
        let callsite_symbol = symbol(callsite.clone());
        self.add_asm_symbolic(directive_globl(callsite_symbol.clone()));
        self.add_asm_symbolic(format!("{}:", callsite_symbol.clone()));            
        
        ValueLocation::Relocatable(RegGroup::GPR, callsite)
    }
    
    fn emit_call_near_r64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation {
        trace!("emit: call {}", func);
        unimplemented!()
    }
    
    fn emit_call_near_mem64(&mut self, callsite: String, func: &P<Value>) -> ValueLocation {
        trace!("emit: call {}", func);
        unimplemented!()
    }
    
    fn emit_ret(&mut self) {
        trace!("emit: ret");
        
        let asm = format!("ret");
        self.add_asm_ret(asm);
    }
    
    fn emit_push_r64(&mut self, src: &P<Value>) {
        trace!("emit: push {}", src);
        
        let (reg, id, loc) = self.prepare_reg(src, 5 + 1);
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("pushq {}", reg);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                rsp => vec![]
            },
            hashmap!{
                id => vec![loc],
                rsp => vec![]
            },
            false
        )
    }
    
    fn emit_push_imm32(&mut self, src: i32) {
        trace!("emit: push {}", src);
        
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("pushq {}", src);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                rsp => vec![]
            },
            hashmap!{
                rsp => vec![]
            },
            false
        )
    }
    
    fn emit_pop_r64(&mut self, dest: &P<Value>) {
        trace!("emit: pop {}", dest);
        
        let (reg, id, loc) = self.prepare_reg(dest, 4 + 1);
        let rsp = self.prepare_machine_reg(&x86_64::RSP);
        
        let asm = format!("popq {}", reg);
        
        self.add_asm_inst(
            asm,
            hashmap!{
                id => vec![loc.clone()],
                rsp => vec![]
            },
            hashmap!{
                rsp => vec![]
            },
            false
        )        
    }    
}

fn create_emit_directory() {
    use std::fs;    
    match fs::create_dir(AOT_EMIT_DIR) {
        Ok(_) => {},
        Err(_) => {}
    }    
}

pub fn emit_code(fv: &mut MuFunctionVersion, vm: &VM) {
    use std::io::prelude::*;
    use std::fs::File;
    use std::path;
    
    let funcs = vm.funcs().read().unwrap();
    let func = funcs.get(&fv.func_id).unwrap().read().unwrap();

    let compiled_funcs = vm.compiled_funcs().read().unwrap();
    let cf = compiled_funcs.get(&fv.id()).unwrap().read().unwrap();

    let code = cf.mc.as_ref().unwrap().emit();

    // create 'emit' directory
    create_emit_directory();

    let mut file_path = path::PathBuf::new();
    file_path.push(AOT_EMIT_DIR);
    file_path.push(func.name().unwrap().to_string() + ".s");
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create emission file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };

    match file.write_all(code.as_slice()) {
        Err(why) => panic!("couldn'd write to file {}: {}", file_path.to_str().unwrap(), why),
        Ok(_) => println!("emit code to {}", file_path.to_str().unwrap())
    }
}

pub fn emit_context(vm: &VM) {
    use std::path;
    use std::fs::File;
    use std::io::prelude::*;
    use rustc_serialize::json;
    
    debug!("---Emit VM Context---");
    create_emit_directory();
    
    let mut file_path = path::PathBuf::new();
    file_path.push(AOT_EMIT_DIR);
    file_path.push(AOT_EMIT_CONTEXT_FILE);
    
    let mut file = match File::create(file_path.as_path()) {
        Err(why) => panic!("couldn't create context file {}: {}", file_path.to_str().unwrap(), why),
        Ok(file) => file
    };
    
    // bss
    {
        // put globals into bss section
        file.write_fmt(format_args!("\t.bss\n")).unwrap();
        
        let globals = vm.globals().read().unwrap();
        for global in globals.values() {
            debug!("emit global: {}", global);
            let (size, align) = {
                let alloc_ty = {
                    match global.v {
                        Value_::Global(ref ty) => ty,
                        _ => panic!("expected a global")
                    }
                };
                
                debug!("getting type: {:?}", alloc_ty);
                let ty_info = vm.get_backend_type_info(alloc_ty.id());
                (ty_info.size, ty_info.alignment)
            };
            
            file.write_fmt(format_args!("\t{}\n", directive_globl(symbol(global.name().unwrap())))).unwrap();
            file.write_fmt(format_args!("\t{}\n", directive_comm(symbol(global.name().unwrap()), size, align))).unwrap();
            file.write("\n".as_bytes()).unwrap();
        }
    }
    
    // data
    // serialize vm
    trace!("start serializing vm");
    {
        let serialize_vm = json::encode(&vm).unwrap();
        
        file.write("\t.data\n".as_bytes()).unwrap();
        
        let vm_symbol = symbol("vm".to_string());
        file.write_fmt(format_args!("{}\n", directive_globl(vm_symbol.clone()))).unwrap();
        let escape_serialize_vm = serialize_vm.replace("\"", "\\\"");
        file.write_fmt(format_args!("\t{}: .asciz \"{}\"", vm_symbol, escape_serialize_vm)).unwrap();
        file.write("\n".as_bytes()).unwrap();
    }
    
    // main_thread
//    let primordial = vm.primordial.read().unwrap();
//    if primordial.is_some() {
//        let primordial = primordial.as_ref().unwrap();
//    }
    
    debug!("---finish---");
}

fn directive_globl(name: String) -> String {
    format!(".globl {}", name)
}

fn directive_comm(name: String, size: ByteSize, align: ByteSize) -> String {
    format!(".comm {},{},{}", name, size, align)
}

#[cfg(target_os = "linux")]
pub fn symbol(name: String) -> String {
    name
}

#[cfg(target_os = "macos")]
pub fn symbol(name: String) -> String {
    format!("_{}", name)
}

use compiler::machine_code::CompiledFunction;

pub fn spill_rewrite(
    spills: &HashMap<MuID, P<Value>>,
    func: &mut MuFunctionVersion,
    cf: &mut CompiledFunction,
    vm: &VM)
{
    // record code and their insertion point, so we can do the copy/insertion all at once
    let mut spill_code_before: HashMap<usize, Vec<Box<ASMCode>>> = HashMap::new();
    let mut spill_code_after: HashMap<usize, Vec<Box<ASMCode>>> = HashMap::new();

    // iterate through all instructions
    for i in 0..cf.mc().number_of_insts() {
        // find use of any register that gets spilled
        {
            let reg_uses = cf.mc().get_inst_reg_uses(i).to_vec();
            for reg in reg_uses {
                if spills.contains_key(&reg) {
                    // a register used here is spilled
                    let spill_mem = spills.get(&reg).unwrap();

                    // generate a random new temporary
                    let temp_ty = func.context.get_value(reg).unwrap().ty().clone();
                    let temp = func.new_ssa(vm.next_id(), temp_ty).clone_value();

                    // generate a load
                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();
                        codegen.emit_mov_r64_mem64(&temp, spill_mem);

                        codegen.finish_code_sequence_asm()
                    };
                    // record that this load will be inserted at i
                    if spill_code_before.contains_key(&i) {
                        spill_code_before.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_before.insert(i, vec![code]);
                    }

                    // replace register reg with temp
                    cf.mc_mut().replace_tmp_for_inst(reg, temp.id(), i);
                }
            }
        }

        // find define of any register that gets spilled
        {
            let reg_defines = cf.mc().get_inst_reg_defines(i).to_vec();
            for reg in reg_defines {
                if spills.contains_key(&reg) {
                    let spill_mem = spills.get(&reg).unwrap();

                    let temp_ty = func.context.get_value(reg).unwrap().ty().clone();
                    let temp = func.new_ssa(vm.next_id(), temp_ty).clone_value();

                    let code = {
                        let mut codegen = ASMCodeGen::new();
                        codegen.start_code_sequence();
                        codegen.emit_mov_mem64_r64(spill_mem, &temp);

                        codegen.finish_code_sequence_asm()
                    };

                    if spill_code_after.contains_key(&i) {
                        spill_code_after.get_mut(&i).unwrap().push(code);
                    } else {
                        spill_code_after.insert(i, vec![code]);
                    }

                    cf.mc_mut().replace_tmp_for_inst(reg, temp.id(), i);
                }
            }
        }
    }

    // copy and insert the code
    let new_mc = {
        let old_mc = cf.mc.take().unwrap();
        let old_mc_ref : &ASMCode = old_mc.as_any().downcast_ref().unwrap();
        old_mc_ref.rewrite_insert(spill_code_before, spill_code_after)
    };

    cf.mc = Some(new_mc);
}