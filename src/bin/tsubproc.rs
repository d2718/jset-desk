use std::process::Command;

fn main() {
    let out = Command::new("frogs").arg("-v").output();
    println!("{:?}", &out);
}