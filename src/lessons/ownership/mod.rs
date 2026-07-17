pub fn obot() -> &'static str {
    /// Rust Ownership & Memory Management
    ///
    /// Ownership is Rust's most unique feature and it's what allows Rust to make memory
    /// safety guarantees without needing a garbage collector.
    ///
    /// Based on insights shared by Obot (@oboobotenefiok).

    const OWNERSHIP_LESSON: &str = r#"
=== Rust Ownership & Memory Management ===

Ownership is Rust's most unique feature and it's what allows Rust to make 
memory safety guarantees without needing a garbage collector.

Based on insights shared by Obot (@oboobotenefiok).

---

THE THREE RULES OF OWNERSHIP

  1. Each value in Rust has a variable that's called its OWNER
  2. There can only be ONE OWNER at a time
  3. When the owner goes out of scope, the value will be DROPPED


RULE 1 & 2: ONE OWNER AT A TIME (THE "MOVE")

In many languages, assigning one variable to another creates a shallow copy 
or a reference. In Rust, for types that store data on the HEAP (like String), 
assigning one variable to another MOVES the ownership.

Example:

    fn main() {
        let indie_hacker = String::from("Caleb");  // indie_hacker OWNS the string
        
        let underdog_builder = indie_hacker;  // OWNERSHIP MOVES to underdog_builder
        
        // println!("{}", indie_hacker);  
        // ^ ERROR! indie_hacker no longer owns the value.
        
        println!("New owner is: {}", underdog_builder);
    }

TIP: To "fix" a move, use .clone() to create a deep copy of the heap data.
    This gives you a second independent owner. Be careful--cloning heap data 
    can be expensive!

NOTE: This prevents "double free" errors. If both variables were valid 
    without cloning, they would both try to free the same memory when they 
    go out of scope.


RULE 3: FUNCTION SCOPE & LIFETIMES

Variables are only accessible within the block {} they are defined in. 
Once the block ends, the variable is "dropped" (freed from memory).

LOCAL VARIABLE SCOPE:

    fn status() {
        let indie_hacker = String::from("Caleb");  // local to status()
        // indie_hacker exists here
    } 
    // indie_hacker is DROPPED here. It cannot be accessed outside.

LIFETIME OF A VARIABLE:
    The lifetime of a variable is the duration for which it exists in memory.
    Once the function status() ends, the memory for indie_hacker is 
    automatically freed.


STACK vs. HEAP: WHY "MOVE" HAPPENS

Rust behaves differently depending on where data is stored:

  1. HEAP (String::from("...")): 
     These are MOVED because they are dynamically sized and stored on the heap. 
     Copying them would be expensive.

  2. STACK (String Literals "...") (like integers or basic string literals):
     These are COPIED instead of moved because copying them is very fast.
     Simple values with a known size at compile time are stored on the STACK.

Example:

    let x = "Caleb";  // Stack literal
    let y = x;        // Copied, not moved. Both are valid.
    println!("{} and {}", x, y);  // Works fine!


KEY TAKEAWAYS

  ✓ Each value has ONE owner
  ✓ Ownership MOVES when assigning heap data to another variable
  ✓ When owner goes out of scope, value is DROPPED
  ✓ Stack data is COPIED (cheap)
  ✓ Heap data is MOVED (expensive to copy)
  ✓ Use .clone() to explicitly copy heap data

Interactive examples in: examples/ownership/
  • move_error.rs   - See what happens when you use a moved variable
  • move_fixed.rs   - How to use .clone() to keep both variables valid
  • scope_dropped.rs - Visualizing when variables are freed from memory
  • stack_copy.rs   - Understanding why simple types don't "move"

*Credit: Based on explanations by @oboobotenefiok* (https://github.com/oboobotenefiok)
"#;

    OWNERSHIP_LESSON
}
