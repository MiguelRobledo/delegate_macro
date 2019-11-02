# delegate_macro
This macro allows you to delegate methods to members of a struct (or any arbitrary expression) similar to how you delegate methods to base classes in inheritance.

# Motivation
I saw [this proposal](https://github.com/contactomorph/rfcs/blob/delegation/text/0000-delegation-of-implementation.md) and thought it would be pretty neat to have this in Rust. I decided to implement it myself to get a taste of what would be needed and how complex this feature would have to be. As it turns out it's pretty trivial to implement although not possible from within a macro due to the lack of access to trait and function signatures (I ended up using a very hacky method to get it to work).

I'm personally quite partial to the syntax proposed in the previous link. `use (expr) for (list of methods);` within an `impl` block  looks pretty intuitive and clean and it doesn't interfere with any existing syntax because (to my knowledge) you cant have `use` within an `impl` block.

# What is delegation?
Delegation is simply telling someone else to do something for you. In Rust you can do this quite easily:
```rust
impl Trait for InnerStruct { /* ... */ }

impl Trait for Struct {
    fn method(&self) {
        self.inner.method();
    }
}
```
This works just fine but adds a bunch of unnecessary boilerplate (imagine having to do this with 10 methods and you can imagine how it can start becoming tedious).


## Types of delegation
### Full delegation
This delegates all methods defined in the trait to `a` which can be an arbitrary expression.
```rust
#[delegate(use a)]
impl Trait for Struct {}
```

### Partial delegation
You can also explicitly choose which methods you want to delegate and you can implement the rest yourself.
This would be similar to overloading a parent method in an inheritance model.
```rust
#[delegate(use a for x, y)]
impl Trait for Struct {
    fn z() {
        println!("I'm not from x!");
    }
}
```

### Mixed delegation
Similar to partial delegation you can explicitly choose which methods to delegate to but you can also mix and match whichever methods for whichever expressions you want.
```rust
#[delegate(
    use a for x;
    use b for y;
)]
impl Trait for Struct {
    fn z() {
        println!("I'm not from x!");
    }
}
```

# Cool, but why?
This feature would add an extremely ergonomic boilerplate-free syntax for code reuse with composition in a similar way to what you can do with inheritance but without actually having inheritance. Technically, this is merely syntactic sugar but boy is it sweet!

# Can I use this expertly crafted macro in production?
NO, PLEASE FOR THE LOVE OF GOD NO.

This is just a proof of concept. It is very hacky and silly.

# Example
This example "works". The only caveat is that it only works if it's in `src/main.rs`. To be more specific, it works if the traits are defined in `src/main.rs`.
```rust
use delegate_macro::delegate;

trait Hello {
    fn hello(&self);
}

trait Math {
    fn add(&self, x: u32, y: u32);
    fn sub(&self, x: i32, y: i32);
}

struct Calculator;
struct Inner;
struct Base {
    inner: Inner,
}

impl Math for Calculator {
    fn add(&self, x: u32, y: u32) {
        println!("{} + {} = {}", x, y, x + y);
    }

    fn sub(&self, x: i32, y: i32) {
        println!("{} - {} = {}", x, y, x - y);
    }
}

impl Inner {
    fn calc(&self) -> Calculator {
        Calculator {}
    }
}

impl Hello for Inner {
    fn hello(&self) {
        println!("howdy!");
    }
}

// direct delegation of all trait methods
#[delegate(use self.inner)]
impl Hello for Base {}

// indirect delegation
// mixed delegation
#[delegate(
    use self.inner.calc() for add;
    use self.inner for sub;
)]
impl Math for Base {}

// partial delegation
#[delegate(use self.calc() for sub)]
impl Math for Inner {
    fn add(&self, _x: u32, _y: u32) {
        println!("Inner doesn't believe in addition");
    }
}

fn main() {
    let x = Base { inner: Inner {} };

    x.hello(); // this will use Inner's hello
    x.add(2, 3); // this will use Calculator's add
    x.inner.add(7, 5); // this will use Inner's add
    x.sub(2, 3); // this will use Calculators's sub
}
```

### Why doesn't the example work elsewhere
Remember how I mentioned you couldn't get trait or function signatures from a macro? Turns out I lied a bit. You can parse any source file and extract the trait and function signatures. That's why this example will only work if placed in `src/main.rs`. The macro is hardcoded to look at that file for all the trait and function signatures it needs. Obviously this limitation wouldn't exist if this feature was implemented in the compiler.
