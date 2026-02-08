/// Example: Variable Dropping and Scope
/// Inspired by @oboobotenefiok

fn main() {
    status();
    
    // println!("{}", indie_hacker); // ERROR: would not compile here
}

fn status() {
    // indie_hacker is local to this block
    let indie_hacker = String::from("Caleb");
    println!("Inside status: {}", indie_hacker);
} // indie_hacker is DROPPED here. Memory is freed.
