 fn main() {
    println!("Welcome to the Rust Learning Journey!");
   println!("Check the 'topics' folder for explanations and 'examples' for code. \n Explanatios are named README while examples are in SNIPPETS");
   start();
  }
  
  fn start() {
    status();
}

fn status() {
    let indie_hacker = String::from("Caleb");
    
    let underdog_builder = indie_hacker; 
  
    println!("This is the name: {}", underdog_builder);
    
}


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
} // indie_hacker is DROPPED here. Memory is freed
.


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
