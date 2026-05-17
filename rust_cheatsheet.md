# Rust Cheatsheet

## Variable Declarations

```rust
let x = 5;              // immutable binding
let mut y = 5;          // mutable binding
let z: i32 = 5;         // explicit type
const MAX: u32 = 100;   // constant (compile-time)
static COUNTER: i32 = 0; // static (runtime, global)
```

## Primitive Types

```rust
i8, i16, i32, i64, i128, isize  // signed integers
u8, u16, u32, u64, u128, usize  // unsigned integers
f32, f64                        // floating point
bool                            // true / false
char                            // single Unicode character
str, String                     // string types
```

## Functions (Routines)

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // return without 'return' keyword
}

fn no_return() {
    println!("Hello"); // no return type = ()
}

fn early_return() -> i32 {
    return 42;
}

// Closures (anonymous functions)
let add = |x, y| x + y;
let result = add(2, 3);
```

## Ownership & Borrowing

```rust
let s1 = String::from("hello");
let s2 = s1;              // move (s1 no longer valid)

let s1 = String::from("hello");
let s2 = &s1;             // immutable borrow
let s3 = &s1;             // multiple immutable borrows OK

let mut s1 = String::from("hello");
let s2 = &mut s1;         // mutable borrow (only one at a time)
```

## Collections

```rust
// Vec: dynamic array
let mut v = Vec::new();
let v = vec![1, 2, 3];
v.push(4);
let first = &v[0];
let first = v.get(0);  // Option<&T>

// HashMap: key-value store
use std::collections::HashMap;
let mut map = HashMap::new();
map.insert("key", "value");
map.get("key");

// VecDeque: double-ended queue
use std::collections::VecDeque;
let mut deque = VecDeque::new();
deque.push_back(1);
deque.push_front(2);
deque.pop_back();
deque.pop_front();
```

## Control Flow

```rust
if x > 5 {
    println!("big");
} else if x == 5 {
    println!("five");
} else {
    println!("small");
}

// if as expression
let result = if x > 5 { "big" } else { "small" };
```

## Loops

```rust
// infinite loop
loop {
    break; // exit loop
}

// while loop
while x < 10 {
    x += 1;
}

// for loop (iterating)
for i in 0..5 {  // 0, 1, 2, 3, 4 (exclusive end)
    println!("{}", i);
}

for i in 0..=5 {  // 0, 1, 2, 3, 4, 5 (inclusive end)
    println!("{}", i);
}

for item in &vec {  // iterate by reference
    println!("{}", item);
}

for item in vec {  // iterate by value (consumes vec)
    println!("{}", item);
}

for (i, item) in vec.iter().enumerate() {
    println!("index: {}, value: {}", i, item);
}
```

## Modules & Organization

```rust
// file: src/lib.rs or src/main.rs
mod player {
    pub struct Player {
        pub x: i32,
        pub y: i32,
    }

    fn private_fn() {}  // not visible outside module

    pub fn public_fn() {} // visible outside module
}

// use to bring into scope
use player::Player;

// absolute path
crate::player::Player;

// relative path
player::Player;
```

File structure:
```
src/
  main.rs       // entry point
  player.rs     // defines module 'player'
  world.rs      // defines module 'world'
```

In `main.rs`:
```rust
mod player;  // load player.rs
mod world;   // load world.rs
```

## Pattern Matching

```rust
// match expression
match value {
    1 => println!("one"),
    2 | 3 => println!("two or three"),
    4..=6 => println!("four to six"),
    _ => println!("other"),
}

// destructuring
let (x, y) = (1, 2);

if let Some(x) = some_option {
    println!("value: {}", x);
}
```

## Structs & Enums

```rust
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 0, y: 0 };

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

## Error Handling

```rust
// Option: Some(T) or None
fn find(vec: &[i32], target: i32) -> Option<usize> {
    for (i, &val) in vec.iter().enumerate() {
        if val == target { return Some(i); }
    }
    None
}

// Result: Ok(T) or Err(E)
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// ? operator (propagate error)
fn safe_divide() -> Result<i32, String> {
    let result = divide(10, 2)?;
    Ok(result * 2)
}

// unwrap (panic if Err/None)
let x = result.unwrap();

// unwrap_or (default value)
let x = option.unwrap_or(0);
```

## Traits

```rust
trait Drawable {
    fn draw(&self);
}

struct Circle {
    radius: f32,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("drawing circle");
    }
}

// using trait
fn render(shape: &dyn Drawable) {
    shape.draw();
}
```

## Lifetime Annotations

```rust
// 'a is a lifetime parameter
fn borrow<'a>(x: &'a String) -> &'a str {
    &x[0..1]
}

struct Container<'a> {
    data: &'a str,
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("expected panic");
    }
}

// Run tests: cargo test
```

## Macros

```rust
println!("Hello, {}!", name);  // print with newline
print!("Hello");               // print without newline
vec![1, 2, 3];                 // create vector
assert_eq!(a, b);              // assert equality
```

## Common Methods

```rust
// String
let s = String::from("hello");
s.len();
s.chars();
s.to_uppercase();
s.trim();

// Vec
let mut v = vec![1, 2, 3];
v.len();
v.push(4);
v.pop();
v.iter();
v.remove(0);
v.contains(&1);

// Slice
let slice = &v[1..3];
```

## Quick Reference: Mutability

```rust
let x = 5;              // immutable (cannot change)
let mut x = 5;          // mutable (can change)
&x                      // immutable borrow
&mut x                  // mutable borrow (exclusive)
```

## Useful Derives

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}
```

- `Debug`: printable with `{:?}` format
- `Clone`: can be cloned
- `Copy`: copied instead of moved (only for small types)
- `PartialEq`, `Eq`: equality operators
- `Default`: has default value
