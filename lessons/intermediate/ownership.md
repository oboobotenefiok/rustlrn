---
title: "Rust Ownership & Memory Management"
level: "intermediate"
estimated_time: "20 min"
prerequisites:
  - "Variables"
  - "Functions"
  - "Stack vs Heap"
tags:
  - "ownership"
  - "memory"
  - "core"
  - "move"
  - "borrow"
author: "Obot (@oboobotenefiok)"
difficulty: 3
---

# Rust Ownership & Memory Management

Ownership is Rust's most unique feature, and it's what allows Rust to make memory safety guarantees without needing a garbage collector.

> Based on insights shared by **Obot (@oboobotenefiok)**.

## The Three Rules of Ownership

1. Each value in Rust has a variable that's called its **owner**.
2. There can only be **one owner** at a time.
3. When the owner goes out of scope, the value is **dropped**.

---

## Rule 1 & 2: One Owner at a Time (The Move)

In many languages, assigning one variable to another creates a shallow copy or a reference. In Rust, for types that store data on the **heap** (like `String`), assigning one variable to another **moves ownership**.

### Example

```rust
fn main() {
    let indie_hacker = String::from("Caleb"); // indie_hacker OWNS the string

    let underdog_builder = indie_hacker; // Ownership MOVES

    // println!("{}", indie_hacker);
    // ERROR: indie_hacker no longer owns the value.

    println!("New owner is: {}", underdog_builder);
}
```

> **Tip:** To keep both variables, use `.clone()` to create a deep copy of the heap data.
>
> ```rust
> let a = String::from("Rust");
> let b = a.clone();
> ```
>
> Be aware that cloning heap data can be expensive.

> **Why does Rust do this?**
>
> This prevents **double-free** errors. If two variables owned the same heap allocation, both would attempt to free the same memory when they went out of scope.

---

## Rule 3: Function Scope & Lifetimes

Variables are only accessible within the block (`{}`) where they are defined.

Once the block ends, the variable is **dropped**, and its memory is automatically freed.

### Local Variable Scope

```rust
fn status() {
    let indie_hacker = String::from("Caleb");
    // indie_hacker exists here
}
// indie_hacker is dropped here.
```

### Lifetime of a Variable

A variable's **lifetime** is the period during which it exists in memory.

When `status()` finishes executing, `indie_hacker` is automatically freed.

---

## Stack vs Heap: Why Moves Happen

Rust behaves differently depending on where data is stored.

### Heap Data (`String::from(...)`)

Heap-allocated values are **moved** because copying them would require duplicating heap memory, which is relatively expensive.

### Stack Data (Integers, Booleans, String Literals, etc.)

Values with a known size at compile time are stored on the **stack**.

These implement the `Copy` trait, so assignments create inexpensive copies instead of moving ownership.

### Example

```rust
fn main() {
    let x = "Caleb"; // &'static str (Copy)
    let y = x;       // Copied, not moved

    println!("{} and {}", x, y);
}
```

---

## Key Takeaways

- Every value has **one owner**.
- Heap data is **moved** by default.
- When an owner goes out of scope, the value is **dropped**.
- Stack data is usually **copied**.
- Heap data can be duplicated explicitly with `.clone()`.
- Rust's ownership model prevents memory leaks and double-free errors without a garbage collector.

---

## Interactive Examples

### Example 1: Move Error

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;

    println!("{}", s1); // ERROR
}
```

### Example 2: Using `clone()`

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();

    println!("{} and {}", s1, s2);
}
```

### Example 3: Stack Copy

```rust
fn main() {
    let x = 5;
    let y = x; // Copied, not moved

    println!("x = {}, y = {}", x, y);
}
```

---

## Practice Exercise

Modify the following code so that it compiles:

```rust
fn main() {
    let word = String::from("Rust");
    let reference = &word; // Borrow instead of taking ownership

    println!("{}", word);
}
```

<details>
<summary>Hint</summary>

A reference (`&T`) borrows a value without becoming its owner.

</details>

---

## Credit

Based on explanations by **@oboobotenefiok**.
