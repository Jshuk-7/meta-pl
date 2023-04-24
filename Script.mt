struct Car {
    make: String,
    model: String,
    year: i32,
}

impl Car {
    proc new(make: String, model: String, year: i32): Car {
        return Car { make: make, model: model, year: year };
    }
}

struct Person {
    name: String,
    age: i32,
}

impl Person {
    proc new(): Person {
        let person: Person = Person { name: "Jackson", age: 22 };
        return person;
    }
}

proc main(): i32 {
    let car: Car = Car { make: "Toyota", model: "Camry", year: 2023 };
    car.year = 2010;

    if car.year == 2010 {
        while car.year < 2023 {
            car.year = car.year + 1;
        }
    }

    for year in 2010..2024 {
        let new_car: Car = Car::new("Honda", "Accord", year);
    }

    return 0;
}