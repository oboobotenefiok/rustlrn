# Rust Ownership & Memory Management

Ownership is Rust's most unique feature and it's what allows Rust to make memory safety guarantees without needing a garbage collector.

This documentation is based on insights shared by **Obot (@oboobotenefiok)**.

## The Three Rules of Ownership

1.  Each value in Rust has a variable that’s called its **owner**.
2.  There can only be **one owner** at a time.
3.  When the owner goes out of scope, the value will be **dropped** (freed from memory).

---

## Rule 1 & 2: One Owner at a Time (The "Move")

In many languages, assigning one variable to another creates a shallow copy or a reference. In Rust, for types that store data on the **heap** (like `String`), assigning one variable to another **moves** the ownership.

### Example: Move vs. Error
When you move a value, the original variable is no longer valid.

```rust
fn main() {
    let indie_hacker = String::from("Caleb"); // indie_hacker owns the string
    
    let underdog_builder = indie_hacker; // OWNERSHIP MOVES to underdog_builder
    
    // println!("{}", indie_hacker); 
    // ^ ERROR! indie_hacker no longer owns the value.
    
    println!("New owner is: {}", underdog_builder);
}
```

> [!TIP]
> **To "fix" a move**, you can use `.clone()`. This creates a deep copy of the data on the heap, giving you a second independent owner. This is used when you actually need two copies of the data, but be careful—cloning heap data can be expensive!

> [!NOTE]
> This prevents "double free" errors. If both variables were valid without cloning, they would both try to free the same memory when they go out of scope.

---

## Rule 3: Function Scope & Lifetimes

Variables are only accessible within the block `{}` they are defined in. Once the block ends, the variable is "dropped".

### Local Variable Scope
If a variable is defined inside a function, it is local to that function.

```rust
fn status() {
    let indie_hacker = String::from("Caleb"); // local to status()
    // indie_hacker exists here
} 
// indie_hacker is DROPPED here. It cannot be accessed outside.
```

### Lifetime of a Variable
The lifetime of a variable is the duration for which it exists in memory. Once the function `status()` ends, the memory for `indie_hacker` is automatically freed.

---

## Stack vs. Heap: Why "Move" Happens

Rust behaves differently depending on where data is stored:

1.  **Heap (`String::from("...")`)**: These are moved because they are dynamically sized and stored on the heap. Copying them would be expensive.
2.  **Stack (String Literals `"..."`)**: Simple values with a known size at compile time (like integers or basic string literals) are stored on the **stack**. These are **copied** instead of moved because copying them is very fast.

```rust
let x = "Caleb"; // Stack literal
let y = x;       // Copied, not moved. Both are valid.
println!("{} and {}", x, y); // Works fine!
```


 ## Interactive Examples

Check the `examples/ownership/` folder for runnable code snippets:

- **[move_error.rs](https://github.com/Spectra010s/rustlrn/blob/main/examples/ownership/move_error.rs)**: [Intentional Error] See what happens when you use a moved variable.
- **[move_fixed.rs](https://github.com/Spectra010s/rustlrn/blob/main/examples/ownership/move_fixed.rs)**: [Fixed] How to use `.clone()` to keep both variables valid.
- **[scope_dropped.rs](https://github.com/Spectra010s/rustlrn/blob/main/examples/ownership/scope_dropped.rs)**: Visualizing when variables are freed from memory.
- **[stack_copy.rs](https://github.com/Spectra010s/rustlrn/blob/main/examples/ownership/stack_copy.rs)**: Understanding why simple types don't "move."

---
*Credit: Based on explanations by @oboobotenefiok* [(@oboobotenefiok)](https://github.com/oboobotenefiok)
