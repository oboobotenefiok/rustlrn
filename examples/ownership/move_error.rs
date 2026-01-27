/// [INTENTIONAL ERROR] Example: The "Move" Rule in Rust Ownership
/// Inspired by @oboobotenefiok
/// 
/// This file will NOT compile. It demonstrates what happens when you try to 
/// use a variable after its value has been moved.

fn main() {
    // 1. indie_hacker owns the String "Caleb"
    let indie_hacker = String::from("Caleb");
    
    // 2. OWNERSHIP MOVES from indie_hacker to underdog_builder
    let underdog_builder = indie_hacker; 
    
    // 3. ERROR! indie_hacker no longer owns the data.
    println!("This will throw error? {}", indie_hacker);
    
    println!("New owner is: {}", underdog_builder);
}
