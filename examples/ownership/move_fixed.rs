/// [FIXED] Example: Handling Moved Values via Cloning
/// Inspired by @oboobotenefiok
/// 
/// This file WILL compile. It demonstrates how to keep the original variable
/// valid by creating a deep copy (clone) of the data.

fn main() {
    let indie_hacker = String::from("Caleb");
    
    // Use .clone() to create a deep copy on the heap.
    // Now both variables own their own distinct "Caleb" string.
    let underdog_builder = indie_hacker.clone(); 
    
    // Both are valid!
    println!("Original: {}", indie_hacker);
    println!("Clone: {}", underdog_builder);
}
