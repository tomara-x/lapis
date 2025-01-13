> yeah, cause when i think "fun", i think "lapis"

this is an interactive interpreter for [FunDSP](https://github.com/SamiPerttu/fundsp). it allows you to experiment/play without needing to compile your code.

if you notice something incorrect, missing, or confusing, please open an issue to tell me about it, or fix it in a pull request if you can.

## wasm
there's a wasm version here: https://tomara-x.github.io/lapis/

execute `set_out_device(0,0);` for audio output to work

## limitations
- you don't have the rust compiler looking over your shoulder
- this isn't rust, you have a very small subset of the syntax
- for functions that accept [`Shape`](https://docs.rs/fundsp/latest/fundsp/shape/trait.Shape.html) as input, `Adaptive` and `ShapeFn` aren't supported
- no closures and therefore none of the functions that take closures as input (yet)
- no `break` or `continue` in loops
- `input()` and file i/o Wave methods won't work in the wasm version

## deviations
- every nodes is wrapped in a `Net`, it's all nets (﻿🌍﻿ 🧑‍🚀﻿ 🔫﻿ 🧑‍🚀﻿)
- i can't support `map`. as a workaround we have `f`. which takes a str argument and outputs that function wrapped in a node. you don't get to define custom functions (at runtime), but at least you get a bunch of basic ones that can then be stitched together like other nodes. see this match for a list of supported functions: https://github.com/tomara-x/lapis/blob/2cace59742819337d414430914c77dd6f225ce74/src/eval/nets.rs#L852
- mutability is ignored. everything is mutable
- type annotations are ignored. types are inferred (`f32`, `Net`, `Vec<f32>`, `bool`, `NodeId`, `Arc<Wave>`, `Shared`, `Sequencer`, `EventId`, `Source`,)
- the `.play()` method for graphs allows you to listen to the graph directly. (graph has to have 0 inputs and 1 or 2 outputs)
- the `input()` node has 2 outputs (left and right) representing the input from mic
- all number variables are f32, even if you type it as `4` it's still `4.0`
- for functions that accept floats you can just type `3` and it's parsed as a float.
- when a function takes an integer or usize, if you type it as a literal integer, then they are parsed to the corresponding type. otherwise (a variable or an expression) they are evaluated as floats then cast to the needed type
- an expression ending in a semicolon, like `variable;`, `2 + 2;`, `lowpass();`, or `[x, x+1, x+2];` will print that expression's value. for Net, Wave, Sequencer, Shared, NodeId, EventId, it will print info about them.
- everything is global. nothing is limited to scope except for the loop variable in for loops
- [`Meter`](https://docs.rs/fundsp/latest/fundsp/dynamics/enum.Meter.html) modes Peak and Rms are actually passed cast f32 not f64

## what's supported

- all functions in the [hacker32 module](https://docs.rs/fundsp/latest/fundsp/hacker/index.html)
    - except for:
        branchi, busi, pipei, stacki, sumi, (and f versions), biquad_bank,
        envelope, envelope2, envelope3, envelope_in (and lfo), fdn, fdn2,
        multitap, multitap_linear, feedback2, flanger, map, oversample,
        phaser, resample, resynth, shape_fn, snoop, unit, update, var_fn

- all functions in the [math module](https://docs.rs/fundsp/latest/fundsp/math/index.html)
    - except for: ease_noise, fractal_ease_noise, hash1, hash2, identity

<details><summary>some f32 methods</summary>
<p>

- floor
- ceil
- round
- trunc
- fract
- abs
- signum
- copysign
- div_euclid
- rem_euclid
- powi
- powf
- sqrt
- exp
- exp2
- ln
- log
- log2
- log10
- cbrt
- hypot
- sin
- cos
- tan
- asin
- acos
- atan
- sinh
- cosh
- tanh
- asinh
- acosh
- atanh
- atan2
- recip
- to_degrees
- to_radians
- max
- min

</p>
</details>

- all the functions in the [sound module] (https://docs.rs/fundsp/latest/fundsp/sound/index.html)

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
x += 1;
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
**vectors**
<details><summary>deviations</summary>
<p>

- when writing vectors you write them as you would an array literal. `let arr = [1,2,3];` instead of `let arr = vec![1,2,3];`
- only `push`, `pop`, `insert`, `remove`, `resize`, `clear`, `len`, `clone`, `first`, `last`, and `get` are supported
- `pop` and `remove` don't return a value, they just remove

</p>
</details>

```rust
let v1 = [1,2,4,6];
let v2 = v1; // identical to let v2 = v1.clone();
v1;
// [1.0, 2.0, 4.0, 6.0]
v2;
// [1.0, 2.0, 4.0, 6.0]
v1[0] = 42;
let f = v1.first();
f;
// 42.0
v1.pop();
v1;
// [42.0, 2.0, 4.0]
v2;
// [1.0, 2.0, 4.0, 6.0]
v1.resize(10,0.1);
v1;
// [42.0, 2.0, 4.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
v2;
// [1.0, 2.0, 4.0, 6.0]
```
[**Net**](https://docs.rs/fundsp/latest/fundsp/net/struct.Net.html)

<details><summary>deviations</summary>
<p>

- `node`, `node_mut`, `wrap`, `wrap_id`, `check`, `backend`, `has_backend` aren't supported
- `.play_backend()` method allows you to play the backend of a net while still being able to edit that same net and commit changes to it. it should only be called once for any given net. the net has to be stored in a variable, have 0 inputs, and 2 outputs
- you can't use the `ids` method directly, but you can use `net.ids().nth(n)`

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
> [!IMPORTANT]
> nets are moved
> ```rust
> let x = sine();
> let y = dc(220) >> x;
> // x is no longer usable here as it's been moved into y
> ```
> but you can avoid this by cloning
> ```rust
> let x = sine();
> let y = dc(220) >> x.clone();
> // x is still usable here
> ```

**tick**
```rust
let net = mul(10);
net.tick([4]); // prints [40.0]
let i = [6];
let o = [];
net.tick(i, o); // o is now [60.0]
```
**shared/var**
```rust
let s = shared(440);
let g = var(s) >> sine();
g.play();
s.set(220);
s.set(s.value() + 42);
```
[**Wave**](https://docs.rs/fundsp/latest/fundsp/wave/struct.Wave.html)

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

[**Sequencer**](https://docs.rs/fundsp/latest/fundsp/sequencer/struct.Sequencer.html)

<details><summary>deviations</summary>
<p>

- backend returns a SequencerBackend wrapped in a net. this way it can be used anywhere were a net can be used
- Sequencer itself can't be played (or `tick`ed). you can either call `play_backend` on it, or `play` on its backend (or a graph containing the backend)
- methods `has_backend`, `replay_events`, and `time` aren't supported
- times are all f32 cast as f64
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
**drop**
```rust
// calling drop on any variable will drop that value
let f = 40;
f + 2;
// 42.0
f.drop();
f; // prints nothing
```

**keyboard shortcuts**

you can bind snippets of code to keyboard shortcuts. keys follow the [egui key names](https://docs.rs/egui/0.30.0/src/egui/data/key.rs.html#310), and modifiers `ctrl`, `shift`, `alt`, and `command` are supported

```rust
"ctrl+shift+a" = "
    // statements
";

"shift+a" = "
    // statements
";

"a" = "
    // statements
";

// reassign to an empty string to remove the key binding
"shift+a" = "";
```

shortcuts can be enabled/disabled using the "keys" toggle at the top of the ui

note: always define the more specific shortcuts (more modifiers) involving the same key before the less specific ones, so `ctrl+shift+a` then `ctrl+a` and `shift+a` then `a`

**device selection**

`list_in_devices` and `list_out_devices` will print an indexed list of hosts and the devices within them. you can use the indexes with `set_in_device` and `set_out_device` to select the devices lapis uses
```rust
list_in_devices();
// input devices:
// 0: Jack:
//     0: Ok("cpal_client_in")
//     1: Ok("cpal_client_out")
// 1: Alsa:
//     0: Ok("pipewire")
//     1: Ok("default")
//     2: Ok("sysdefault:CARD=sofhdadsp")
list_out_devices();
// output devices:
// 0: Jack:
//     0: Ok("cpal_client_in")
//     1: Ok("cpal_client_out")
// 1: Alsa:
//     0: Ok("pipewire")
//     1: Ok("default")

set_in_device(1, 2); // selects host 1 (alsa), device 2 (sysdef...) from the input devices list
set_out_device(1, 0); // selects host 1 (alsa), device 0 (pipewire) from the output list
```

## building

- install rust: https://www.rust-lang.org/tools/install
- on linux you need `libjack-dev` and `libasound2-dev` (`jack-devel` and `alsa-lib-devel` on void)
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
- crossbeam_channel https://github.com/crossbeam-rs/crossbeam
- eframe_template https://github.com/emilk/eframe_template

## license

lapis is free and open source. all code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### your contributions

unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

