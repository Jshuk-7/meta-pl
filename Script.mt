struct Program {
    name: String
}

proc process(p: Program): bool {
    return p.name == "Hello";
}

proc main() {
    let p = Program { name: "Hello world" };
    process(p);
}