use std::collections::{HashMap, HashSet};

use inkwell::values::{FunctionValue, InstructionOpcode};
use tracing::{warn};
use z3::ast::{Bool, Int, BV};
use z3::Solver;

use crate::codegen_basic_block::codegen_basic_block;
use crate::get_var_name::get_var_name;


pub fn get_forward_edges(function: &FunctionValue) -> HashMap<String, HashSet<String>> {
    let mut all_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let mut node_edges = HashSet::new();
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        if let Some(terminator) = bb.get_terminator() {
            let opcode = terminator.get_opcode();
            let num_operands = terminator.get_num_operands();
            match &opcode {
                InstructionOpcode::Return => {
                    // NO-OP
                }
                InstructionOpcode::Br => {
                    if num_operands == 1 {
                        // Unconditional branch
                        let successor_basic_block = terminator.get_operand(0).unwrap().right().unwrap();
                        let successor_basic_block_name = String::from(successor_basic_block.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name));
                    } else if num_operands == 3 {
                        // Conditional branch
                        let successor_basic_block_1 = terminator.get_operand(1).unwrap().right().unwrap();
                        let successor_basic_block_name_1 = String::from(successor_basic_block_1.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name_1));
                        let successor_basic_block_2 = terminator.get_operand(2).unwrap().right().unwrap();
                        let successor_basic_block_name_2 = String::from(successor_basic_block_2.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name_2));
                    } else {
                        warn!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                    }
                }
                InstructionOpcode::Switch => {
                    for operand in 0..num_operands {
                        if operand % 2 == 1 {
                            let successor_basic_block = terminator.get_operand(operand).unwrap().right().unwrap();
                            let successor_basic_block_name = String::from(successor_basic_block.get_name().to_str().unwrap());
                            node_edges.insert(String::from(successor_basic_block_name));
                        }
                    }
                }
                InstructionOpcode::IndirectBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Invoke => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CallBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Resume => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchSwitch => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CleanupRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Unreachable => {
                    // NO-OP
                }
                _ => {
                    warn!("Opcode {:?} is not supported as a terminator for edge generation", opcode);
                }
            }
        } else {
            warn!("\tNo terminator");
        }
        all_edges.insert(basic_block_name, node_edges);
    }
    return all_edges;
}


fn get_backward_edges(function: &FunctionValue) -> HashMap<String, HashSet<String>> {
    let all_forward_edges = get_forward_edges(function);
    let mut all_backward_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        all_backward_edges.insert(basic_block_name, HashSet::new());
    }
    for (source, dests) in all_forward_edges {
        for dest in dests {
            if let Some(reverse_dests) = all_backward_edges.get_mut(&dest) {
                reverse_dests.insert(source.clone());
            }
        }
    }
    return all_backward_edges;
}


fn forward_topological_sort(function: &FunctionValue) -> Vec<String> {
    let forward_edges = get_forward_edges(function);
    let backward_edges = get_backward_edges(function);
    let mut sorted = Vec::new();
    let mut unsorted = Vec::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        unsorted.push(basic_block_name);
    }
    let num_nodes = unsorted.len();

    let mut indegrees = HashMap::new();
    for node in &unsorted {
        if let Some(reverse_dests) = backward_edges.get(&node.clone()) {
            let mut indegree = 0;
            for _j in 0..reverse_dests.len() {
                indegree += 1;
            }
            indegrees.insert(node, indegree);
        }
    }

    while sorted.len() < num_nodes {
        let mut next_node: Option<String> = None;
        for node in &unsorted {
            if let Some(indegree) = indegrees.get(&node.clone()) {
                if (*indegree) == 0 {
                    indegrees.insert(node, -1);
                    next_node = Some(node.to_string());
                    sorted.push(node.to_string());
                    if let Some(dests) = forward_edges.get(&node.clone()) {
                        for dest in dests.into_iter() {
                            if let Some(prev_indegree) = indegrees.get_mut(&dest.clone()) {
                                *prev_indegree = *prev_indegree - 1;
                            }
                        }
                    }
                }
            }
        }
        match next_node {
            Some(..) => (),
            None => {
                warn!("CFG is cyclic which is not supported");
                break;
            }
        }
    }
    return sorted;
}


fn backward_topological_sort(function: &FunctionValue) -> Vec<String> {
    let mut sorted = forward_topological_sort(function);
    sorted.reverse();
    return sorted;
}


pub fn codegen_function(function: &FunctionValue, solver: &Solver, namespace: &str) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    let forward_edges = get_forward_edges(&function);
    let backward_edges = get_backward_edges(&function);
    let backward_sorted_nodes = backward_topological_sort(&function);

    for node in backward_sorted_nodes {
        codegen_basic_block(node, &forward_edges, &backward_edges, function, solver, namespace);
    }

    // constrain int inputs
    for input in function.get_params() {
        // TODO: Support other input types
        if input.get_type().to_string().eq("\"i1\"") {
            continue;
        } else if input.get_type().to_string().eq("\"i32\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, namespace));
            let min_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
            let max_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
            solver
                .assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        } else if input.get_type().to_string().eq("\"i64\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, namespace));
            let min_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MIN.into(), 64), true);
            let max_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MAX.into(), 64), true);
            solver
                .assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        }  else {
            warn!("Currently unsuppported type {:?} for input parameter", input.get_type().to_string())
        }
    }
}