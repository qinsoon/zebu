use ast::ptr::P;
use ast::op::{BinOp, CmpOp, AtomicRMWOp};
use ast::types::*;

pub type WPID  = usize;
pub type MuID  = usize;
pub type MuTag = &'static str;
pub type Address = usize; // TODO: replace this with Address(usize)

#[derive(Debug)]
pub struct MuFunction {
    pub fn_name: MuTag,
    pub sig: P<MuFuncSig>,
    pub entry: MuTag,
    pub blocks: Vec<(MuTag, Block)>
}

#[derive(Debug)]
pub struct Block {
    pub label: MuTag,
    pub content: Option<BlockContent>
}

impl Block {
    pub fn new(label: MuTag) -> Block {
        Block{label: label, content: None}
    }
}

#[derive(Debug)]
pub struct BlockContent {
    pub args: Vec<P<TreeNode>>,
    pub body: Vec<P<TreeNode>>,
    pub keepalives: Option<Vec<P<TreeNode>>>    
}

#[derive(Clone, Debug)]
/// always use with P<TreeNode>
pub struct TreeNode {
    pub v: TreeNodeKind,
    pub children: Vec<P<TreeNode>>,
}

impl TreeNode {
    pub fn new_value(v: P<Value>) -> P<TreeNode> {
        P(TreeNode{v: TreeNodeKind::Value(v), children: vec![]})
    }
    
    pub fn new_inst(v: Instruction) -> P<TreeNode> {
        P(TreeNode{v: TreeNodeKind::Instruction(v), children: vec![]})
    }
}

#[derive(Clone, Debug)]
pub enum TreeNodeKind {
    Value(P<Value>),
    Instruction(Instruction),
}

/// always use with P<Value>
#[derive(Clone, Debug)]
pub enum Value {
    SSAVar(SSAVar),
    Constant(MuConstant)
}

impl Value {
    pub fn new_ssa(v: SSAVar) -> P<Value> {
        P(Value::SSAVar(v))
    }
    
    pub fn new_constnat(v: MuConstant) -> P<Value> {
        P(Value::Constant(v))
    }
}

#[derive(Clone, Debug)]
pub struct SSAVar {
    pub id: MuID,
    pub tag: MuTag,
    pub ty: P<MuType_>
}

#[derive(Clone, Debug)]
pub struct MuConstant{
    pub ty: P<MuType_>, 
    pub val: Constant
}

#[derive(Clone, Debug)]
pub enum Constant {
    Int(usize, usize),
    IRef(P<MuType_>, Address),
    FloatV(f32),
    DoubleV(f64),
    VectorV(Vec<Constant>),
    FuncRefV(Address),
    UFuncRefV(Address)
}

#[derive(Clone, Debug)]
pub enum Instruction {
    NonTerm(NonTermInstruction),
    Term(Terminal)
}

#[derive(Clone, Debug)]
pub enum Terminal {
    Return(Vec<P<TreeNode>>),
    ThreadExit,
    Throw(Vec<P<TreeNode>>),
    TailCall(CallData),
    Branch1(Destination),
    Branch2{
        cond: P<TreeNode>,
        true_dest: Destination,
        false_dest: Destination
    },
    Watchpoint{ // Watchpoint NONE ResumptionData
                //   serves as an unconditional trap. Trap to client, and resume with ResumptionData
                // Watchpoint (WPID dest) ResumptionData
                //   when disabled, jump to dest
                //   when enabled, trap to client and resume
        id: Option<WPID>,
        disable_dest: Option<Destination>,
        resume: ResumptionData
    }, 
    WPBranch{
        wp: WPID, 
        disable_dest: Destination,
        enable_dest: Destination
    },
    Call{
        data: CallData,
        resume: ResumptionData
    },
    SwapStack{
        stack: P<TreeNode>,
        is_exception: bool,
        args: Vec<P<TreeNode>>,
        resume: ResumptionData
    },
    Switch{
        cond: P<TreeNode>,
        default: Destination,
        branches: Vec<(P<Constant>, Destination)>
    },
    ExnInstruction{
        inner: NonTermInstruction,
        resume: ResumptionData
    }
}

#[derive(Clone, Debug)]
pub enum NonTermInstruction {
    Assign{
        left: Vec<P<TreeNode>>,
        right: Expression_
    },

    Fence(MemoryOrder),
}

#[derive(Clone, Debug)]
pub enum Expression_ {
    BinOp(BinOp, P<TreeNode>, P<TreeNode>), 
    CmpOp(CmpOp, P<TreeNode>, P<TreeNode>),
    
    // yields the constant value
    Constant(P<Constant>),
    
    // yields a tuple of results from the call
    ExprCall{
        data: CallData,
        is_abort: bool, // T to abort, F to rethrow
    },
    
    // yields the memory value
    Load{
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder
    },
    
    // yields nothing
    Store{
        is_iref: bool,
        mem_loc: P<Value>,
        order: MemoryOrder        
    },
    
    // yields pair (oldvalue, boolean (T = success, F = failure))
    CmpXchg{
        is_iref: bool, // T for iref, F for ptr
        is_strong: bool,
        success_order: MemoryOrder,
        fail_order: MemoryOrder,
        mem_loc: P<TreeNode>,
        expected_value: P<TreeNode>,
        desired_value: P<TreeNode>
    },
    
    // yields old memory value
    AtomicRMW{
        is_iref: bool, // T for iref, F for ptr
        order: MemoryOrder,
        op: AtomicRMWOp,
        mem_loc: P<TreeNode>,
        value: P<TreeNode> // operand for op
    },
    
    // yields a reference of the type
    New(P<MuType_>),
    
    // yields an iref of the type
    AllocA(P<MuType_>),
    
    // yields ref
    NewHybrid{    // hybrid type, var part length
        ty: P<MuType_>, 
        var_len: P<TreeNode>
    },  
    
    // yields iref
    AllocAHybrid{
        ty: P<MuType_>, 
        var_len: P<TreeNode>
    },
    
    // yields stack ref
    NewStack{
        func: P<TreeNode>
    },
    
    // yields thread reference
    NewThread{
        stack: P<TreeNode>,
        args: Vec<P<TreeNode>>
    },
    
    // yields thread reference (thread resumes with exceptional value)
    NewThreadExn{
        stack: P<TreeNode>,
        exn: P<TreeNode>
    },
    
    // yields frame cursor
    NewFrameCursor(P<TreeNode>), // stack
    
    GetIRef(P<TreeNode>),
    
    GetFieldIRef{
        base: P<TreeNode>, // iref or ptr
        index: P<Constant>
    },
    
    GetElementIRef{
        base: P<TreeNode>,
        index: P<TreeNode>
    },
    
    ShiftIRef{
        base: P<TreeNode>,
        offset: P<TreeNode>
    },
    
    GetVarPartIRef(P<TreeNode>),
    
//    PushFrame{
//        stack: P<Value>,
//        func: P<Value>
//    },
//    PopFrame{
//        stack: P<Value>
//    }
}

#[derive(Copy, Clone, Debug)]
pub enum MemoryOrder {
    NotAtomic,
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst
}

#[derive(Copy, Clone, Debug)]
pub enum CallConvention {
    Mu,
    Foreign(ForeignFFI)
}

#[derive(Copy, Clone, Debug)]
pub enum ForeignFFI {
    C
}

#[derive(Clone, Debug)]
pub struct CallData {
    pub func: P<TreeNode>,
    pub args: Vec<P<TreeNode>>,
    pub convention: CallConvention
}

#[derive(Clone, Debug)]
pub struct ResumptionData {
    pub normal_dest: Destination,
    pub exn_dest: Destination
}

#[derive(Clone, Debug)]
pub enum DestArg {
    Normal(P<TreeNode>),
    Freshbound(usize)
}

#[derive(Clone, Debug)]
pub struct Destination {
    pub target: MuTag,
    pub args: Vec<DestArg>
}