use std::{collections::VecDeque};
use std::env;

use llvm_ir::{Module, Function, Instruction};
use llvm_ir_analysis::{ModuleAnalysis, FunctionAnalysis, CFGNode};
use rustc_demangle::demangle;
use tracing::debug;
use z3::{
    ast::{self, Ast, Bool, Int},
    SatResult,
};
use z3::{Config, Context, Solver};


fn parse_control_flow_graph(func_analysis: &FunctionAnalysis) -> () {
    //! Parses the llvm-ir-analysis function_analysis object for each basic block
    let cfg = func_analysis.control_flow_graph();

    let mut blocks = Vec::<CFGNode>::new();
    
    let mut stack = VecDeque::<CFGNode>::new();
    stack.extend(cfg.succs(cfg.entry()));
    match stack[0] {
        CFGNode::Block(name) => {
            let preds = cfg.preds(name);
            debug!("Start preds:");
            for pred in preds {
                debug!("{:?}", pred);
            }
        },
        _ => debug!("\tReturn block"),
    }

    while stack.len() > 0 {
        let current_block = stack.pop_front().unwrap();
        if !blocks.contains(&current_block) {
            blocks.push(current_block);
            match current_block {
                CFGNode::Block(name) => {
                    debug!("\tvalue: {}", name);
                    stack.extend(cfg.succs(name));
                },
                _ => debug!("\tReturn block"),
            }
        }
    }
}

fn parse_instruction(instr: &Instruction) -> () {

    match instr {
        Instruction::Add(add) => {
            println!("\tAdd operation: {:?}", add)
        },
        Instruction::Mul(mul) => {
            println!("\tMul operation: {:?}", mul)
        }
        unknown_opp => {
            debug!("\tUnknown operation: {:?}", unknown_opp);
        }
    }
}


fn backward_symbolic_execution(func: &Function) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    println!("\tBasic Blocks:");
    for bb in &func.basic_blocks {
        println!("\t\t{:?}", bb.name);
        for instr in &bb.instrs {
            println!("\t\t\t{:?}", instr.to_string());
            parse_instruction(instr);
        }
        println!("\t\t\t{:?}", bb.term);
    }
}


fn print_file_functions(module: &Module) -> () {
    //! Iterates through all functions in the file and prints the demangled name

    println!("Functions in {:?}:", module.name);
    for func in &module.functions {
        println!("\t{:?} == {:?}", demangle(&func.name.as_str()).to_string(), func.name.to_string());
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_name = String::from("tests/hello_world.bc");
    if args.len() > 1 {
        // Use custom user file
        file_name = args[1].to_string();
    }

    let module = Module::from_bc_path(file_name).unwrap();
    
    print_file_functions(&module);

    let ma = ModuleAnalysis::new(&module);
    let func = module.get_func_by_name("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E").unwrap();
    let fa = ma.fn_analysis("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E");
    
    debug!("Backward Symbolic Execution in {:?}", demangle("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E"));
    backward_symbolic_execution(func);
    parse_control_flow_graph(fa);
}
