struct Program {
    name: String,
    instruction_count: i32,
}

proc new_program(): Program {
    let program: Program = Program { name: "Script.mt", instruction_count: 1 };
    return program;
}

proc main() {
    let program: Program = new_program();
}