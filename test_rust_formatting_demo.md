# Rust Code Formatting Demo

This document demonstrates the automatic formatting of Rust code with:
- **Bold keywords** (fn, let, if, struct, etc.)
- *Italic comments* (both line comments and inline comments)

## Example 1: Simple Function

```rust
// This function prints a greeting
fn greet(name: &str) {
    // Print the greeting message
    println!("Hello, {}!", name); // inline comment
}
```

## Example 2: Struct Definition

```rust
// Define a Person struct
pub struct Person {
    pub name: String,    // person's name
    pub age: u32,        // person's age
}

// Implementation block
impl Person {
    // Constructor function
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
    
    // Method to check if person is adult
    pub fn is_adult(&self) -> bool {
        self.age >= 18  // check age threshold
    }
}
```

## Example 3: Control Flow

```rust
fn main() {
    let x = 42;  // initialize variable
    
    // Check if x is positive
    if x > 0 {
        println!("Positive");
    } else if x < 0 {
        println!("Negative");  // won't execute
    } else {
        println!("Zero");
    }
    
    // Loop example
    for i in 0..5 {
        println!("{}", i);  // print each number
    }
}
```

## Example 4: Pattern Matching

```rust
// Enum definition
enum Color {
    Red,
    Green,
    Blue,
}

// Match expression
fn color_name(color: Color) -> &'static str {
    match color {
        Color::Red => "red",    // red variant
        Color::Green => "green", // green variant
        Color::Blue => "blue",   // blue variant
    }
}
```

All Rust keywords should appear in **bold**, and all comments should appear in *italic*.
