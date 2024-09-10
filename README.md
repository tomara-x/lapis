> yeah, cause when i think "fun", i think "lapis"

lists marked non_exhaustive may be incomplete. if you notice something incorrect, missing, or confusing, please open an issue to tell me about it, or fix it in a pull request if you can.

## limitations
#[non_exhaustive]
- you don't have the rust compiler looking over your shoulder
- this isn't rust, you have a very small subset of the syntax
- for functions that accept [`Shape`](https://docs.rs/fundsp/0.19.0/fundsp/shape/trait.Shape.html) as input, `Adaptive` and `ShapeFn` aren't supported
- no closures and therefore none of the functions that take closures as input (yet)
- no `break` or `continue` in loops
- Net methods like `remove` don't return a node

## todo
#[non_exhaustive, help_welcome]
- Net methods aren't checked and will panic if misused
- float methods (some of them)
- vector methods (same)
- Sequencer
- i/o device selection (in the settings window)
- input node abstraction
- atomic synth
- TODO marks in eval/nets.rs
- optimize egui stuff (high cpu use if large amount of text is in buffer)

## deviations
#[non_exhaustive]
- mutability is ignored. everything is mutable
- type annotations are ignored. types are inferred (`f32`, `Net`, `Vec<f32>`, `bool`, or `NodeId`)
- when writing vectors you write them as you would an array literal. `let arr = [2, 50.4, 4.03];` instead of `vec![2, 50.4, 4.03]`
- the `.play()` method for graphs allow you to listen to the graph directly. (graph has to have 0 inputs and 1 or 2 outputs)
- `.play_backend()` allows you to play the backend of a net while still being able to edit that same net and commit changes to it. it should only be called once for any given net. (net has to be stored in a variable, have 0 inputs, and 2 outputs)
- all number variables are f32, even if you type it as `4` it's still `4.0`
- for functions that accept floats you can just type `3` and it's parsed as a float.
- when a function takes an integer or usize, those are parsed to the corresponding type, so you can't use a variable there (since those are all floats), it has to be an integer literal
- a statement with just a variable name `variable;` will print that variable's value (or call .display() for graphs) same for expressions `2 + 2;` will print 4
- everything is global. nothing is limited to scope except for the loop variable in for loops
- [`Meter`](https://docs.rs/fundsp/0.19.0/fundsp/dynamics/enum.Meter.html) modes peak and rms are actually passed cast f32 not f64

## what's supported
#[non_exhaustive]

assignment
```rust
let x = 9;
let y = 4 + 4 * x - (3 - x * 4);
let osc3 = sine() & mul(3) >> sine() & mul(0.5) >> sine(); 
let f = lowpass_hz(1729, 0.5);
let out = osc3 >> f;
```
reassignment
```rust
let x = 42;
x = 56;         // x is still a number. this works
x = sine();     // x is a number can't assign an audio node (x is still 56.0)
let x = sine(); // x is now a sine()
```
if conditions
```rust
let x = true && 2 < 8;
let y = 3 % 2 == 1;
if x {
    // red pill
} else if y {
    // blue pill
} else {
    // get rambunctious
}
```
for loops
```rust
// with ranges
for i in 0..5 {
    let x = i + 3;
    x;
}
let node = Net::new(0,0);
for i in 0..=9 {
    node = node | dc(i);
}
// over array elements
let arr = [1,2,3];
for i in [4,6,8] {
    for j in arr {
        i * j;
    }
}
```
blocks
```rust
// to execute multiple statements at once, put them in a block
{
    let x = 0;
    for i in 0..3 {
        x = x + i;
    }
    x;
}
```
net methods
```rust
let net = Net::new(0,2);
let id = net.push(sine_hz(440));
net.connect_output(id,0,0);
net.connect_output(id,0,1);
net.play(); // not a method for Net (see deviations)
```
tick
```rust
let net = mul(10);
net.tick([4]); // prints [40.0]
let in = [6];
let out = [];
net.tick(in, out); // out is now [60.0]
```
shared/var
```rust
let s = shared(440);
let g = var(s) >> sine();
g.play();
s.set(220);
```


## building

- install rust: https://www.rust-lang.org/tools/install
- install egui dependencies: https://github.com/emilk/egui#demo
- on linux also install libjack-dev
- clone lapis
```
git clone https://github.com/tomara-x/lapis.git
```
- build it
```
cd lapis
cargo run --release
```

## thanks

- fundsp https://github.com/SamiPerttu/fundsp
- egui https://github.com/emilk/egui
- syn https://github.com/dtolnay/syn
- cpal https://github.com/rustaudio/cpal

## license

lapis is free and open source. all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### your contributions

unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

