use ast::ir::*;
use compiler::frame::*;
use runtime::ValueLocation;

use std::ops;
use std::collections::HashMap;

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

pub struct CompiledFunction {
    pub func_id: MuID,
    pub func_ver_id: MuID,
    pub temps: HashMap<MuID, MuID>, // assumes one temperary maps to one register
    
    // not emitting this
    pub mc: Option<Box<MachineCode>>,
    
    pub frame: Frame,
    pub start: ValueLocation,
    pub end: ValueLocation
}

const CF_SERIALIZE_FIELDS : usize = 6;

impl Encodable for CompiledFunction {
    fn encode<S: Encoder> (&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("CompiledFunction", CF_SERIALIZE_FIELDS, |s| {
            try!(s.emit_struct_field("func_id",     0, |s| self.func_id.encode(s)));
            try!(s.emit_struct_field("func_ver_id", 1, |s| self.func_ver_id.encode(s)));
            try!(s.emit_struct_field("temps",       2, |s| self.temps.encode(s)));
            try!(s.emit_struct_field("frame",       3, |s| self.frame.encode(s)));
            try!(s.emit_struct_field("start",       4, |s| self.start.encode(s)));
            try!(s.emit_struct_field("end",         5, |s| self.end.encode(s)));
            
            Ok(())
        })
    }
}

impl Decodable for CompiledFunction {
    fn decode<D: Decoder>(d: &mut D) -> Result<CompiledFunction, D::Error> {
        d.read_struct("CompiledFunction", CF_SERIALIZE_FIELDS, |d| {
            let func_id = 
                try!(d.read_struct_field("func_id",     0, |d| Decodable::decode(d)));
            let func_ver_id = 
                try!(d.read_struct_field("func_ver_id", 1, |d| Decodable::decode(d)));
            let temps = 
                try!(d.read_struct_field("temps",       2, |d| Decodable::decode(d)));
            let frame = 
                try!(d.read_struct_field("frame",       3, |d| Decodable::decode(d)));
            let start = 
                try!(d.read_struct_field("start",       4, |d| Decodable::decode(d)));
            let end =
                try!(d.read_struct_field("end",         5, |d| Decodable::decode(d)));
            
            Ok(CompiledFunction{
                func_id: func_id,
                func_ver_id: func_ver_id,
                temps: temps,
                mc: None,
                frame: frame,
                start: start,
                end: end
            })
        })
    }
}

impl CompiledFunction {
    pub fn mc(&self) -> &Box<MachineCode> {
        match self.mc {
            Some(ref mc) => mc,
            None => panic!("trying to get mc from a compiled function. 
                But machine code is None (probably this compiled function is restored from
                boot image and mc is thrown away)")
        }
    }
    
    pub fn mc_mut(&mut self) -> &mut Box<MachineCode> {
        match self.mc {
            Some(ref mut mc) => mc,
            None => panic!("no mc found from a compiled function")
        }
    }
}

pub trait MachineCode {
    fn trace_mc(&self);
    fn trace_inst(&self, index: usize);
    
    fn emit(&self) -> Vec<u8>;
    
    fn number_of_insts(&self) -> usize;
    
    fn is_move(&self, index: usize) -> bool;
    fn is_using_mem_op(&self, index: usize) -> bool;
    
    fn get_succs(&self, index: usize) -> &Vec<usize>;
    fn get_preds(&self, index: usize) -> &Vec<usize>;
    
    fn get_inst_reg_uses(&self, index: usize) -> &Vec<MuID>;
    fn get_inst_reg_defines(&self, index: usize) -> &Vec<MuID>;
    
    fn get_ir_block_livein(&self, block: &str) -> Option<&Vec<MuID>>;
    fn get_ir_block_liveout(&self, block: &str) -> Option<&Vec<MuID>>;
    fn set_ir_block_livein(&mut self, block: &str, set: Vec<MuID>);
    fn set_ir_block_liveout(&mut self, block: &str, set: Vec<MuID>);
    
    fn get_all_blocks(&self) -> &Vec<MuName>;
    fn get_block_range(&self, block: &str) -> Option<ops::Range<usize>>;
    
    fn replace_reg(&mut self, from: MuID, to: MuID);
    fn set_inst_nop(&mut self, index: usize);
}
