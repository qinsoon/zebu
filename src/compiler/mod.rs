extern crate hprof;

use ast::ir::*;
use vm::VM;
use std::cell::RefCell;

pub mod passes;
pub mod backend;
pub mod frame;
pub mod machine_code;

pub use compiler::passes::CompilerPass;

pub struct Compiler<'vm> {
    policy: RefCell<CompilerPolicy>,
    vm: &'vm VM
}

impl <'vm> Compiler<'vm> {
    pub fn new(policy: CompilerPolicy, vm: &VM) -> Compiler {
        Compiler{
            policy: RefCell::new(policy),
            vm: vm
        }
    }

    pub fn compile(&self, func: &mut MuFunctionVersion) {
        trace!("{:?}", func);
        
        // FIXME: should use function name here (however hprof::enter only accept &'static str)
        let _p = hprof::enter("Function Compilation");

        let ref mut passes = self.policy.borrow_mut().passes;

        for pass in passes.iter_mut() {
            let _p = hprof::enter(pass.name());

            pass.execute(self.vm, func);

            drop(_p);
        }

        drop(_p);

        hprof_print_timing(hprof::profiler().root());

        func.set_compiled();
    }

    pub fn get_policy(&self) -> &RefCell<CompilerPolicy> {
        &self.policy
    }
}

pub struct CompilerPolicy {
    pub passes: Vec<Box<CompilerPass>>
}

impl CompilerPolicy {
    pub fn new(passes: Vec<Box<CompilerPass>>) -> CompilerPolicy {
        CompilerPolicy{passes: passes}
    }
}

impl Default for CompilerPolicy {
    fn default() -> Self {
        let mut passes : Vec<Box<CompilerPass>> = vec![];
        passes.push(Box::new(passes::Inlining::new()));
        // ir level passes
        passes.push(Box::new(passes::DefUse::new()));
        passes.push(Box::new(passes::TreeGen::new()));
        passes.push(Box::new(passes::GenMovPhi::new()));
        passes.push(Box::new(passes::ControlFlowAnalysis::new()));
        passes.push(Box::new(passes::TraceGen::new()));

        // compilation
        passes.push(Box::new(backend::inst_sel::InstructionSelection::new()));
        passes.push(Box::new(backend::reg_alloc::RegisterAllocation::new()));

        // machine code level passes
        passes.push(Box::new(backend::peephole_opt::PeepholeOptimization::new()));
        passes.push(Box::new(backend::code_emission::CodeEmission::new()));

        CompilerPolicy{passes: passes}
    }
}

// rewrite parts of the hprof crates to print via log (instead of print!())
use self::hprof::ProfileNode;
use std::rc::Rc;

fn hprof_print_timing(root: Rc<ProfileNode>) {
    info!("Timing information for {}:", root.name);
    for child in &*root.children.borrow() {
        hprof_print_child(child, 2);
    }
}

fn hprof_print_child(this: &ProfileNode, indent: usize) {
    let mut indent_str = "".to_string();
    for _ in 0..indent {
        indent_str += " ";
    }

    let parent_time = this.parent
        .as_ref()
        .map(|p| p.total_time.get())
        .unwrap_or(this.total_time.get()) as f64;
    let percent = 100.0 * (this.total_time.get() as f64 / parent_time);
    if percent.is_infinite() {
        info!("{}{name} - {calls} * {each} = {total} @ {hz:.1}hz",
        indent_str,
        name  = this.name,
        calls = this.calls.get(),
        each = Nanoseconds((this.total_time.get() as f64 / this.calls.get() as f64) as u64),
        total = Nanoseconds(this.total_time.get()),
        hz = this.calls.get() as f64 / this.total_time.get() as f64 * 1e9f64
        );
    } else {
        info!("{}{name} - {calls} * {each} = {total} ({percent:.1}%)",
        indent_str,
        name  = this.name,
        calls = this.calls.get(),
        each = Nanoseconds((this.total_time.get() as f64 / this.calls.get() as f64) as u64),
        total = Nanoseconds(this.total_time.get()),
        percent = percent
        );
    }
    for c in &*this.children.borrow() {
        hprof_print_child(c, indent+2);
    }
}

// used to do a pretty printing of time
struct Nanoseconds(u64);

use std::fmt;
impl fmt::Display for Nanoseconds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1_000 {
            write!(f, "{}ns", self.0)
        } else if self.0 < 1_000_000 {
            write!(f, "{:.1}us", self.0 as f64 / 1_000.)
        } else if self.0 < 1_000_000_000 {
            write!(f, "{:.1}ms", self.0 as f64 / 1_000_000.)
        } else {
            write!(f, "{:.1}s", self.0 as f64 / 1_000_000_000.)
        }
    }
}
