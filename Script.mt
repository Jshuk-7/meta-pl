struct Person {
    name: String,
    age: i32,
}

proc main() {
    let person: Person = Person { name: "Jack", age: 22 };
    person.name = "Fred";
    
    person.age = 12;

    let name = person.name;
    let age = person.age;
}