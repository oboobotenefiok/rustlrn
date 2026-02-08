/// Example: Stack-based Data (Copy vs Move)
/// Inspired by @oboobotenefiok

fn main() {
    // Simple values with a fixed size (integers, bools, chars, &str)
    // are stored on the STACK. They implement the 'Copy' trait.
    
    let x = "Caleb"; // Simple string literal (&str)
    let y = x;       // Copied automatically, not moved.
    
    println!("Stack values are copied: x = {}, y = {}", x, y);
    
    let a = 10;
    let b = a; // Copied
    println!("Integers too: a = {}, b = {}", a, b);
}
