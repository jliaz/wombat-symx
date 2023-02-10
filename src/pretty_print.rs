pub fn print_file_functions(module: &InkwellModule) -> () {
    //! Iterates through all functions in the file and prints the demangled name
    debug!("Functions in {:?}:", module.get_name());
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        debug!("\t{:?} == {:?}", demangle(current_function.get_name().to_str().unwrap()).to_string(), current_function.get_name());
        next_function = current_function.get_next_function();
    }
    debug!("");
}


pub fn pretty_print_function(function: &FunctionValue) -> () {
    debug!("Number of Nodes: {}", function.count_basic_blocks());
    debug!("Arg count: {}", function.count_params());
    // No local decl available to print
    debug!("Basic Blocks:");
    for bb in function.get_basic_blocks() {
        debug!("\tBasic Block: {:?}", bb.get_name().to_str().unwrap());
        debug!("\t\tis_cleanup: {:?}", is_panic_block(&bb));
        let mut next_instruction = bb.get_first_instruction();
        let has_terminator = bb.get_terminator().is_some();

        while let Some(current_instruction) = next_instruction {
            debug!("\t\tStatement: {:?}", current_instruction.to_string());
            next_instruction = current_instruction.get_next_instruction();
        }

        if has_terminator {
            debug!("\t\tLast statement is a terminator");
        } else {
            debug!("\t\tNo terminator");
        }
    }
    debug!("");

    let first_basic_block = function.get_first_basic_block().unwrap();
    debug!("Start node: {:?}", first_basic_block.get_name().to_str().unwrap());
    let forward_edges = get_forward_edges(function);
    let successors = forward_edges.get(first_basic_block.get_name().to_str().unwrap()).unwrap();
    for successor in successors {
        debug!("\tSuccessor to start node: {:?}", successor);
    }
}