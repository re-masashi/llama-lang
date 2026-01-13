trait Converter {
    fn convert<U>(self, other: U) -> U;
}

struct MyType {}

impl Converter for MyType {
    fn convert<U>(self, other: U) -> U {
        other
    }
}
