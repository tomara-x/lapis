> yeah, cause when i think "fun", i think "lapis"

lists marked non_exhaustive may be incomplete. if you notice something incorrect, missing, or confusing, please open an issue to tell me about it, or fix it in a pull request if you can.

## limitations
#[non_exhaustive]
- you don't have the rust compiler looking over your shoulder
- this isn't rust, you have a very small subset of the syntax
- for functions that accept [`Shape`](https://docs.rs/fundsp/0.19.0/fundsp/shape/trait.Shape.html) as input, `Adaptive` and `ShapeFn` aren't supported
- no closures and therefore none of the functions that take closures as input (yet)
- no `break` or `continue` in loops

## todo
#[non_exhaustive, help_welcome]
- Net methods aren't checked and will panic if misused
- vector methods (some)
- i/o device selection (in the settings window)
- input node abstraction
- atomic synth
- TODO marks in eval/nets.rs
- optimize egui stuff (high cpu use if large amount of text is in buffer)

## deviations
#[non_exhaustive]
- mutability is ignored. everything is mutable
- type annotations are ignored. types are inferred (`f32`, `Net`, `Vec<f32>`, `bool`, `NodeId`, `Arc<Wave>`, `Shared`, Sequencer, EventId,)
- when writing vectors you write them as you would an array literal. `let arr = [2, 50.4, 4.03];` instead of `vec![2, 50.4, 4.03]`
- the `.play()` method for graphs allows you to listen to the graph directly. (graph has to have 0 inputs and 1 or 2 outputs)
- all number variables are f32, even if you type it as `4` it's still `4.0`
- for functions that accept floats you can just type `3` and it's parsed as a float.
- when a function takes an integer or usize, if you type it as a literal integer, then they are parsed to the corresponding type. otherwise (a variable or an expression) they are evaluated as floats then cast to the needed type
- a statement with just a variable name `variable;` will print that variable's value (or call .display() for graphs) same for expressions `2 + 2;` will print 4
- everything is global. nothing is limited to scope except for the loop variable in for loops
- [`Meter`](https://docs.rs/fundsp/0.19.0/fundsp/dynamics/enum.Meter.html) modes peak and rms are actually passed cast f32 not f64

## what's supported
#[non_exhaustive]

<details><summary>all functions in hacker32 except for</summary>
<p>

- branchf, branchi, busf, busi, pipef, pipei, stackf, stacki, sumf, sumi
- envelope, envelope2, envelope3, envelope_in,
- lfo, lfo2, lfo3, lfo_in
- fdn, fdn2
- multitap, multitap_linear
- feedback2
- flanger
- map
- oversample
- phaser
- resample
- resynth
- shape_fn
- snoop
- unit
- update
- var_fn

</p>
</details>


<details><summary>all functions in the math module except for</summary>
<p>

- ease_noise
- fractal_ease_noise
- hash1
- hash2
- identity

</p>
</details>

**assignment**
```rust
let x = 9;
let y = 4 + 4 * x - (3 - x * 4);
let osc3 = sine() & mul(3) >> sine() & mul(0.5) >> sine(); 
let f = lowpass_hz(1729, 0.5);
let out = osc3 >> f;
```
**reassignment**
```rust
let x = 42;
x = 56;         // x is still a number. this works
x = sine();     // x is a number can't assign an audio node (x is still 56.0)
let x = sine(); // x is now a sine()
```
**if conditions**
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
**for loops**
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
**blocks**
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
**Net**

<details><summary>deviations</summary>
<p>

- `remove`, `remove_link`, `replace` will work but won't return a node
- `node`, `node_mut`, `wrap`, `wrap_id`, `check`, `backend`, `has_backend` aren't supported
- `.play_backend()` method allows you to play the backend of a net while still being able to edit that same net and commit changes to it. it should only be called once for any given net. the net has to be stored in a variable, have 0 inputs, and 2 outputs

</p>
</details>

```rust
let net = Net::new(0,2);
net.play_backend();
let id = net.push(sine_hz(440));
net.connect_output(id,0,0);
net.connect_output(id,0,1);
net.commit();
```
**tick**
```rust
let net = mul(10);
net.tick([4]); // prints [40.0]
let in = [6];
let out = [];
net.tick(in, out); // out is now [60.0]
```
**shared/var**
```rust
let s = shared(440);
let g = var(s) >> sine();
g.play();
s.set(220);
```
**Wave**

<details><summary>deviations</summary>
<p>

- the `remove` method removes the channel but doesn't return a vec
- the `channel` method returns a cloned vec
- output from methods `channels`, `len`, and `duration` is cast as f32
- `is_empty`, `channel_mut`, `write_wav16`, `write_wav32`, `load_slice`, `load_slice_track` aren't implemented
- methods on `Wave`s can only be called on a stored variable. so you can't say `Wave::zero(2,44100,1).channel(0)` for example. you have to assign the wave to a variable then call the method on that variable
- it's actually an Arc<Wave>, methods are called using Arc::make_mut.
- can't be cloned

</p>
</details>

<details><summary>Arc(Wave)</summary>
<p>

```rust
// waves use Arc. they're cloned when mutated while other references exist
// (Arc::make_mut)

// load a song (keep a system monitor open to watch the memory use)
let wave = Wave::load("song.mp3");

// this doesn't clone the wave, since no other references exist
wave.set_sample_rate(48000);

// no memory use increase. the players use the same copy
let w1 = wavech(wave, 0);
let w2 = wavech(wave, 1);

// this causes the wave to be cloned (one in graphs, and the new edited one here)
wave.set_sample_rate(48000);

// redefining the graphs, dropping the old wave
let w1 = wavech(wave, 0);
let w2 = wavech(wave, 1);

// if you're using `play()`, it has to be called twice for an old graph to be dropped
// since it uses a Slot, which keeps the previous graph for morphing
```

</p>
</details>

```rust
let w = Wave::load("./guidance.wav");   // load from file
w;                                      // prints info about the loaded wave
    //Wave(ch:1, sr:11025, len:1101250, dur:99.88662131519274)

let osc = sine_hz(134) | saw_hz(42);
let s = Wave::render(44100, 1, osc);    // render 1 second of the given graph
s;                                      // print info
    //Wave(ch:2, sr:44100, len:44100, dur:1)
s.save_wav16("awawawa.wav");            // save the wave as a 16-bit wav file
```

**Sequencer**

<details><summary>deviations</summary>
<p>

- backend returns a SequencerBackend wrapped in a net. this way it can be used anywhere were a net can be used
- Sequencer itself can't be played (or `tick`ed). you can either call `play_backend` on it, or `play` on its backend (or a graph containing the backend)
- methods `has_backend`, `replay_events`, and `time` aren't supported
- you can't clone them (and why would you want to?)

</p>
</details>

```rust
let s = Sequencer::new(true, 2);
s;
// Sequencer(outs: 2, has_backend: false)
let b = s.backend();
b;
// Inputs         : 0
// Outputs        : 2
// Latency        : 0.0 samples
// Footprint      : 224 bytes
// Size           : 1
s;
// Sequencer(outs: 2, has_backend: true)
let g = b >> reverb_stereo(30,3,0.8);
g;
// Inputs         : 0
// Outputs        : 2
// Latency        : 0.0 samples
// Footprint      : 224 bytes
// Size           : 2
g.play();
s.push_relative(0, 2, Fade::Smooth, 0.2, 0.1,
	sine_hz(124) | sine_hz(323)
);
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

