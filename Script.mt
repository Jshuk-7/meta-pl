struct Person {
    name: String,
    age: i32,
}

proc do_work(p: Person) {
    p.name = "Jack";
}

proc main() {
    let person: Person = Person { name: "Jack", age: 22 };
    person.age = person.age + 1;

    do_work(person);
}