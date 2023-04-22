proc do_work() {
    let b = 2;
    b = 5;
}

proc main() {
    let a = 10;
    a = 12;

    do_work();
}