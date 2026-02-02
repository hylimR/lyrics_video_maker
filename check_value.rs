use evalexpr::Value;

fn main() {
    let mut v = Value::Int(42);
    if let Value::Int(ref mut i) = v {
        *i = 100;
    }
    println!("{:?}", v);
}
