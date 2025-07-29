# Example

.. some theory ..

.. some snippets ..
```rust
let mut number = 3;
while number > 0 {
    println!("{}!", number);
    number -= 1;
}
println!("Liftoff!");

// While with complex conditions
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

## Mini-Exercises
(Expandable example solutions hidden by default)

**a.** Declare a mutable variable `counter`, initialize it to `0`, and increment it in a loop until it reaches `10`.

```rust,editable
fn main() {
    println!("hello");
}
```

<details>
<summary>Solution</summary>

```rust
let mut counter = 0;
while counter < 10 {
    counter += 1;
    println!("Counter: {}", counter);
}
```
</details>