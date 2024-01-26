use json_parser::parse;

fn main() {
    let json_val = "{\"foo\":[1,2,3]}".as_bytes();
    let parsed = parse(json_val).unwrap();
    println!("{:?}", parsed);
}
