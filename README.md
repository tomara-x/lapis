> yeah, cause when i think "fun", i think "lapis"

this is an interactive interpreter for [FunDSP](https://github.com/SamiPerttu/fundsp). it allows you to experiment/play without needing to compile your code.

for a command line version, see: [codeberg.org/tomara-x/lazurite](https://codeberg.org/tomara-x/lazurite)

if you have questions, come to the [rust audio](https://discord.gg/3hfDXS6hhQ) server (the fundsp channel)

if you notice something incorrect, missing, or confusing, please open an issue to tell me about it, or fix it in a pull request if you can.

## wasm
there's a wasm version here: https://tomara-x.github.io/lapis/

execute `set_out_device(_,_,_,_,_);` for audio output to work

## limitations
- you don't have the rust compiler looking over your shoulder
- this isn't rust, you have a very small subset of the syntax
- for functions that accept [`Shape`](https://docs.rs/fundsp/latest/fundsp/shape/trait.Shape.html) as input, `Adaptive` and `ShapeFn` aren't supported
- no closures and therefore none of the functions that take closures as input (yet)
- `input()`, `sleep()`, and file i/o `Wave` methods won't work in the wasm version

## additions
- the `net.play()` method allows you to listen to an audio net (net must have 0 inputs and outputs equal to output stream channels)
```rust
let g = sine_hz(110) >> pan(0);
g.play();                   // you should hear a 110hz tone
(dc(0) | dc(0)).play();     // back to silence
```
- the `input()` node outputs mic input. `input(0, 1)` is the right and left channels of a stereo mic. while `input(0)` outputs just the first channel
```rust
(input(0, 1) >> reverb_stereo(20,3,0.5)).play();
// you should hear the input from your mic being played back
```

- similar to the functionality of `Snoop` and `Ring`, you can use `bounded` to create a ring buffer
<details><summary>bounded examples</summary>
<p>

```rust
let (i, o) = bounded(4); // channel with capacity 4 (maximum capacity is 1000000)
// the sender and the receiver are both wrapped in nets
i
// Inputs         : 1
// Outputs        : 1
// Latency        : 0.0 samples
// Footprint      : 248 bytes
// Size           : 1
o
// Inputs         : 0
// Outputs        : 1
// Latency        : 0.0 samples
// Footprint      : 248 bytes
// Size           : 1

// tick the input to send samples
// it passes its input through, so it can be placed
// anywhere in an audio graph to monitor the latest n samples
i.tick([1]);
// [1.0]
i.tick([2]);
// [2.0]
i.tick([3]);
// [3.0]
i.tick([4]);
// [4.0]

// this won't make it, the channel is full
i.tick([5]);
// [5.0]

// tick the output to receive samples
o.tick([]);
// [1.0]
o.tick([]);
// [2.0]
o.tick([]);
// [3.0]
o.tick([]);
// [4.0]

// channel is empty
o.tick([]);
// [0.0]

i.tick([1729]);
// [1729.0]
o.tick([]);
// [1729.0]
o.tick([]);
// [0.0]
```

they can also be used for feedback (but buffer() (see below) is better for that)
```rust
let (i, o) = bounded(1);
let f = (o+pass()) >> i;
```

</p>
</details>

- buffer() works similarly to bounded() but it's for situations where you want both ends to be in the audio graph (like a portal)

<details><summary>buffer example</summary>
<p>

```rust
// capacity should be the number of samples that get processed at a time
// (for a playing graph, that's 64 because we use the BlockRateAdapter)
// (for ticking maually, that's 1)
let (i, o) = buffer(64);
let f = (o+pass()) >> i;
```

</p>
</details>

- `rfft` and `ifft` nodes (since using the fft functions directly isn't possible here)
<details><summary>fft example</summary>
<p>

```rust
let len = 512; // window length

// generate a hann window
let win_wave = Wave::new(1, 44100);
for i in 0..len {
    let p = 0.5 + -cos(i/len * TAU)/2;
    win_wave.push(p);
}

// for overlap (note the starting points of 1 and 3)
// they're staring points, not offsets (unlike the fft nodes)
let w0 = wavech_at(win_wave, 0, 0, len, 0);
let w1 = wavech_at(win_wave, 0, len * 0.75, len, 0);
let w2 = wavech_at(win_wave, 0, len * 0.50, len, 0);
let w3 = wavech_at(win_wave, 0, len * 0.25, len, 0);
let window = w0 | w1 | w2 | w3;

// split the input into 4 copies
let input = input(0) >> split::<U4>();

let ft = rfft(len, 0)
       | rfft(len, len * 0.25)
       | rfft(len, len * 0.50)
       | rfft(len, len * 0.75);

let ift = ifft(len, 0)
        | ifft(len, len * 0.25)
        | ifft(len, len * 0.50)
        | ifft(len, len * 0.75);

let real = pass() | sink()
         | pass() | sink()
         | pass() | sink()
         | pass() | sink();

// we only care about the real output of the inverse fft
let ift = ift >> real;

// generate delay times wave (for delaying the bins)
let delay_wave = Wave::new(1, 44100);
for i in 0..len/2 {
    // random numbers from 0 to 500
    let p = rnd1(i) * 500;
    // we want an integer multiple of the window duration (so we don't freq shift)
    let p = p.round() * len;
    // push that duration in seconds instead of samples
    delay_wave.push(p / 44100);
}
// delay each bin by the amount of time in delay_wave
let tmp = (pass() | wavech(delay_wave, 0, 0)) >> tap(0, 15);
// need 8 copies (4 overlaps, real and imaginary each)
let process = Net::new(0,0);
for i in 0..8 {
    process = process | tmp.clone();
}

let g = input * window.clone() >> ft >> process >> ift * window >> join::<U4>() >> pan(0);

g.play();
```

</p>
</details>

## deviations
- this is using my [fork](https://codeberg.org/tomara-x/fundsp) of fundsp
- every nodes is wrapped in a `Net`
- mutability is ignored. everything is mutable
- type annotations are ignored. types are inferred (`f64`, `Net`, `Vec<f32>`, `bool`, `NodeId`, `Arc<Wave>`, `Shared`, `Sequencer`, `EventId`, `Source`, `Arc<AtomicTable>`, `String`,)
- when a function takes an integer or usize, if you type it as a literal integer, then they are parsed to the corresponding type. otherwise (a variable or an expression) they are evaluated as f64 then cast to the needed type
- an expression, like `variable`, `2 + 2`, `lowpass()`, or `[x, x+1, x+2]` will print that expression's value. for `Net`, `Wave`, `Sequencer`, `Shared`, `NodeId`, `EventId`, it will print info about them.
- everything is global. nothing is limited to scope except for the loop variable in for loops

## what's supported

- all functions in the [hacker32 module](https://docs.rs/fundsp/latest/fundsp/hacker/index.html)
    - except for:
        branchi, busi, pipei, stacki, sumi, (and f versions), biquad_bank,
        envelope, envelope2, envelope3, envelope_in (and lfo), fdn, fdn2,
        multitap, multitap_linear, feedback2, map, oversample,
        resample, resynth, shape_fn, snoop, unit, update, var_fn
    - `flanger` and `phaser` are edited to accept modulation as a second input channel rather than a modulation function

- all functions in the [math module](https://docs.rs/fundsp/latest/fundsp/math/index.html)
    - except for: ease_noise, fractal_ease_noise, hash1, hash2, identity
    - i have 2 additional functions ([wrap and mirror](https://codeberg.org/tomara-x/fundsp/src/commit/47b301b7612c3c24eb46ef58dacd52b21077d336/src/math.rs#L762-L774))

<details><summary>some f64 methods</summary>
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

- all functions in the [sound module](https://docs.rs/fundsp/latest/fundsp/sound/index.html)
- [std constants](https://doc.rust-lang.org/std/f64/consts/index.html), `inf`, `-inf`, and `nan`

### assignment
```rust
let x = 9;
let y = 4 + 4 * x - (3 - x * 4);
let osc3 = sine() & mul(3) >> sine() & mul(0.5) >> sine(); 
let f = lowpass_hz(1729, 0.5);
let out = osc3 >> f;
```
### reassignment
```rust
let x = 42;
x = 56;         // x is still a number. this works
x += 1;         // binary assignment works. (+=, -=, *=, /=, and %=)
x = sine();     // x is a number. can't assign an audio node (x is still 57.0)
let x = sine(); // x is now a sine()
```
### if conditions
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
### for loops
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
### vectors
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

// binary assignment isn't supported for indexes
// so you can't do vec[0] += 1;
// you have to do vec[0] = vec[0] + 1;
```
### [Net](https://docs.rs/fundsp/latest/fundsp/net/struct.Net.html)

<details><summary>deviations</summary>
<p>

- `node`, `node_mut`, `wrap`, `check`, `has_backend` aren't supported
- you can't use the `ids` method directly, but you can use `net.ids().nth(n)`

</p>
</details>

```rust
let net = Net::new(0,2);
net.backend().play();
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

### [tick](https://docs.rs/fundsp/latest/fundsp/audionode/trait.AudioNode.html#tymethod.tick)
process one frame of samples through a graph

```rust
let net = pass() | mul(10);
// input vector must have same number of elements as the graph's inputs
net.tick([5, 2]);
// [5.0, 20.0]

// tick returns an array, you can use it normally
let i = [13, 1.2];
let o = net.tick(i);
o;
// [13.0, 12.0]
```
### [shared/var](https://github.com/SamiPerttu/fundsp#atomic-variables)
```rust
let s = shared(440);
let g = var(s) >> sine();
g.play();
s.set(220);
s.set(s.value() + 42);
```
### [Wave](https://docs.rs/fundsp/latest/fundsp/wave/struct.Wave.html)

<details><summary>deviations</summary>
<p>

- the optional loop point argument of wavech and wavech_at can be specified as a number or omitted (or as `Some(n)`)
- the `remove_channel` method removes the channel but doesn't return a vec
- the `channel` method returns a cloned vec
- output from methods `channels` and `len` is cast as f64
- `is_empty`, `channel_mut`, `write_wav16`, `write_wav32`, `load_slice`, `load_slice_track` aren't implemented
- methods (other than filter/filter_latency) can only be called on a stored variable. so you can't say `Wave::zero(2,44100,1).channel(0)` for example. you have to assign the wave to a variable then call the method on that variable
- what's stored in variables is an `Arc<Wave>`. editing methods are called using `Arc::make_mut`, cloning the wave if more than one referece exists. or they can be prefixed with `unsafe_` (e.g. `wave.unsafe_set(...)`) to avoid that (but should you?)
- `clone` clones the arc, not the wave

</p>
</details>

<details><summary>Arc(Wave)</summary>
<p>

```rust
// waves are stored as Arc<Wave>
// the underlying wave is cloned when mutated while other references exist
// (Arc::make_mut) (unless you use the unsafe methods)

// (keep a system monitor open to watch the memory use)
// (or keep an eye on the arc count in the wave info)
// load a song
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

// useless knowledge:
// if you're using `play()`, it has to be called twice for an old graph to be dropped
// since it uses a Slot, which keeps the previous graph for morphing
```

</p>
</details>

```rust
let w = Wave::load("./guidance.wav");   // load from file
w;                                      // prints info about the loaded wave
// Wave(ch:1, sr:11025, len:1101250, dur:99.88662131519274, arcs:1)

let player = wavech(w, 0);              // a player of channel 0 with no looping
let looping_player = wavech(w, 0, 0);   // jumps to sample 0 when it finishes

let osc = sine_hz(134) | saw_hz(42);
let s = Wave::render(44100, 1, osc);    // render 1 second of the given graph
s;                                      // print info
// Wave(ch:2, sr:44100, len:44100, dur:1, arcs:1)
s.save_wav16("awawawa.wav");            // save the wave as a 16-bit wav file
```

### [Sequencer](https://docs.rs/fundsp/latest/fundsp/sequencer/struct.Sequencer.html)

<details><summary>deviations</summary>
<p>

- `.backend()` returns a SequencerBackend wrapped in a net. this way it can be used anywhere a net is usable
- Sequencer itself can't be `play`ed or `tick`ed. do that to its backend (or a graph containing the backend)
- methods `has_backend`, `replay_events`, and `time` aren't supported
- you can't clone frontends (and why would you want to?)
- `.set_loop(start, end)` lets you set loop times in seconds. (0, inf) by default
- `.set_time(t)` jumps to time
- `.set_replay_events(bool)` change whether this sequencer retains past events
- `.clear()` clear all events
- `Sequencer::io(inputs, outputs)` creates a sequencer with specified number of ins and outs (effect sequencer)

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

### [AtomicTable](https://docs.rs/fundsp/latest/fundsp/shared/struct.AtomicTable.html)
```rust
// create atomic table
let table = atomic_table([1,3,1,2]); // anything that evaluates to an array
                                     // (like wave.channel()) is valid here
                                     // but the array must be a power of 2
table.at(0); // index reading
// 1.0
table.set(0,13121); // index writing
table.at(0)
// 13121.0

// you can play them using atomic synth
let table = atomic_table([-0.1, 0, 0.1, 0]);
let frequency = 110;
let g = dc(frequency) >> atomic_synth(table) >> pan(0);
g.play();
table.set(0, 0.); // change the table while it's playing
table.set(1, 0.);
table.set(2, 0.);
table.set(3, 0.);

// atomic_synth can take an extra argument specifying the interpolation type ("nearest" by default)
atomic_synth(table, "linear");
atomic_synth(table, "cubic");

// or using a phase reader, atomic_phase has the same syntax as atomic_synth
let g = ramp_hz(frequency) >> atomic_phase(table, "cubic") >> pan(0);
```

### [phase synth](https://docs.rs/fundsp/latest/fundsp/wavetable/struct.PhaseSynth.html)
phase_synth takes a str argument specifying the wave

(`hammond`, `organ`, `saw`, `soft_saw`, `square`, `triangle`, `sine`)

```rust
// 2 operator phase modulation example
let depth = shared(0);
let modulator = ramp() >> phase_synth("sine");
let carrier = (ramp() + modulator * var(depth)) >> phase_synth("sine");
let g = dc(55) >> split::<U2>() >> (pass() | mul(0.5)) >> carrier >> pan(0);
g.play();
// change modulation depth
depth.set(0.4);
depth.set(1);
depth.set(0);
```


### drop
```rust
// calling drop on any variable will drop that value
let f = 40;
f + 2;
// 42.0
f.drop();
f; // prints nothing
```

### keyboard shortcuts

you can bind snippets of code to keyboard shortcuts. keys follow the [egui key names](https://docs.rs/egui/0.33.0/src/egui/data/key.rs.html#328), and modifiers `ctrl`, `shift`, `alt`, and `command` are supported

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

// starting a shortcut with `!` means this block is evaluated on the release of that shortcut
"!a" = "
    // statements evaluated when `a` is released
";

"!shift+a" = "
    // statements evaluated when `a` is released while shift is held
";

// `@` signs in the assigned string will be substituted with the key name
// e.g. when pressing the f key, `let XOF = 0;` will be evaluated
"f" = "let XO@ = 0;";
```

shortcuts can be enabled/disabled using the "keys" toggle at the top of the ui

note: always define the more specific shortcuts (more modifiers) involving the same key before the less specific ones, so `ctrl+shift+a` then `ctrl+a` and `shift+a` then `a`


### ui things
```rust
// change ui toggles
"quiet" = true;
"keys" = true;
"keys_repeat" = true;

// set zoom factor
zoom_factor(1.5);
```

### device selection

`list_in_devices` and `list_out_devices` will print an indexed list of hosts and the devices within them. you can use the indexes with `set_in_device` and `set_out_device` to select the devices lapis uses

set_in/out_device also accept arguments for specifying the channel count, sample rate, and buffer size of the stream.

default host/device/configs will be used for any argument that evaluates to none (use _ for example)

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

// set_in_device(host_index, device_index, channel_count, sample_rate, buffer_size);

set_in_device(1, 2, _, _, _); // selects host 1 (alsa), device 2 (sysdef...) from the input devices list
set_out_device(1, 0, _, _, _); // selects host 1 (alsa), device 0 (pipewire) from the output list

// to stope a stream you can use
drop_in_stream();
drop_out_stream();
```

### stream info

```rust
// channel count, sample rate, and buffer size of the output stream
out_stream.chan;
out_stream.sr;
out_stream.buffer;    // these are none if the buffer size is set to default

// .. and the input stream
in_stream.chan;
in_stream.sr;
in_stream.buffer;
```

### f

i can't support `map`. as a workaround we have `f`. it takes a str argument and outputs that function wrapped in a node. you don't get to define custom functions (at runtime), but at least you get a bunch of basic ones that can then be stitched together like other nodes

the function definitions are here: [maps.rs](https://codeberg.org/tomara-x/fundsp/src/branch/main/src/maps.rs)

```rust
// this is equivalent to `map(|i: &Frame<f32, U2>| f32::from(i[0] > i[1]))`
let x = f(">");
x;
// Inputs         : 2
// Outputs        : 1
// Latency        : 0.0 samples
// Footprint      : 232 bytes
// Size           : 1
```

<details><summary>function list</summary>
<p>

- rise
- fall
- \>
- <
- ==
- !=
- \>=
- <=
- min
- max
- pow
- rem
- rem_euclid
- rem2 (x % 2)
- log
- bitand (those 5 bitwise functions cast their inputs to integer)
- bitor
- bitxor
- shl
- shr
- lerp
- lerp11
- delerp
- delerp11
- xerp
- xerp11
- dexerp
- dexerp11
- abs
- signum
- floor
- fract
- ceil
- round
- sqrt
- exp
- exp2
- exp10
- exp_m1
- ln_1p
- ln
- log2
- log10
- hypot
- atan2
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
- squared
- cubed
- dissonance
- dissonance_max
- db_amp
- amp_db
- a_weight
- m_weight
- spline
- spline_mono
- softsign
- softexp
- softmix
- smooth3
- smooth5
- smooth7
- smooth9
- uparc
- downarc
- sine_ease
- sin_hz
- cos_hz
- sqr_hz
- tri_hz
- rnd1
- rnd2
- spline_noise
- fractal_noise
- to_pol (takes cartesian, outputs polar)
- to_car (takes polar, outputs cartesian)
- to_deg (takes radians, outputs degrees)
- to_rad (takes degrees, outputs radians)
- recip (1 / x)
- normal (filters inf/-inf/nan)
- wrap (in 0..1 range)
- mirror (in 0..1 range)

</p>
</details>

### additional nodes
 most of the nodes defined in [misc_nodes.rs](https://codeberg.org/tomara-x/fundsp/src/branch/main/src/misc_nodes.rs) are available, but some have slightly different syntax here

```rust
ahr(attack, hold, release)  // an attack-hold-release node (times in seconds)
select(node1, ...)  // takes any number of 0-in 1-out nodes and selects based on the input (index)
fade_select(node1, ...)  // same as select but does crossfading on fractional indexes
seq(node1, ...)  // takes any number of 0-in 1-out nodes. it has 4 inputs
                 // (trigger, index, delay, duration)
                 // pushes an event of index node when trigger is non-zero
shift_reg()  // an 8-output shift register (cascading sample and hold)
             // takes 2 inputs (signal, trigger)
quantizer(Vec<f32>)  // quantizes its input to the given values
                     // (values must be non-negative and in ascending order)
kr(node, n)  // subsamples a node (of any arity) (tick it every n samples)
                   // (there's an additional bool arg, but it's bad for you)
reset(node, duration)  // resets the inner node every duration (node must be 0-ins 1-out)
trig_reset(node)  // takes a 0-ins 1-out node and resets it whenever its input is non-zero
reset_v(node)  // takes a 0-ins 1-out node and resets it every duration (its input)
snh()  // sample and hold node. has 2 inputs (signal, trigger) samples when trigger is non-zero
euclid()  // euclidean rhythm generator. has 4 inputs
          // trigger (step forward when non-zero), length, pulses, rotation
resample1(node)  // resample but only works on 0-in 1-out nodes
bitcrush()  // a bit crusher node. takes 2 inputs (signal, number of steps/2)
gate(duration)  // gate. outputs 1 for duration, 0 after that
t()  // outputs node time (subsampled to ~2ms, cause it uses envelope())
unsteady(Vec<f32>, bool)  // outputs impulses at the given durations
                          // the bool is whether or not to loop
unsteady_nr(Vec<f32>, bool)  // same but isn't affected by resets
unsteady_ramp(Vec<f32>, bool)  // its output goes one integer up every given duration
                               // the bool also sets looping
step(node1, ...)  // steps forward through its nodes when its input is non-zero
                  // a node gets reset when it's selected
                  // nodes must be 0-in 1-out
filter_step(node1, ...)  // same as step but takes 1-in 1-out nodes
                         // and has 2 inputs (input passed to the selected node, trigger)
wave_at(Arc<Wave>) // read from a wave. takes 2 channels (channel, index)
wave_set(Arc<Wave>) // set a value in a wave. takes 3 channels (channel, index, value)
wave_mix(Arc<Wave>) // same but mixes
```

### sliders
you can add sliders in the sliders window and link them to variables (floats and shared variables will work).

you can also create them using lapis code (for saving a setup)
```rust
let min = 0;
let max = 10;
let speed = 0.1;
let step_by = 0;

let x = 0;
let y = shared(1);
add_slider("x", min, max, speed, step_by);
add_slider("y", min, max, speed, step_by);
```

### strings
you can assign strings to variables, do simple substitutions, and evaluate strings
```rust
let str1 = "hola!";

// format will replace as many `$` in the string with the arguments you give it
format("$$$$", 1, 3, 1, 2);
// "1312"

// format can also substitute string arguments, or a mixture of numbers and strings
format("$ = $", "pi", 100);
// "pi = 100"

let x = 4;
let y = 7;
let str2 = format("pi = $, and tau = $", x, y);
str2;
// "pi = 4, and tau = 7"

let n = 1;
let str3 = format("let awa$$ = 1729;", "wawa", n);
// now print str3
str3;
// "let awawawa1 = 1729;"
// you can evaluate this string by doing
eval(str3);
// you can check that variable
awawawa1;
// 1729.0

// you can also use quiet_eval() to avoid printing
quiet_eval("let x = 1729;");
// now try printing x
x;
// 1729.0

// you can load the contents of a file into a string
let str4 = file("../path/to/file.rs");

// replace
let str4 = replace(str4, "awa", "awawawa");

// replacen
let str4 = replacen(str4, "awawawa", "awa", 2);

```

### clear
```rust
// clear all defined keybindings
clear_keys();

// clear sliders
clear_sliders();

// clear all variable bindings
clear_maps();

// all of the above
clear();
```

### time
you can access the time since the unix epoch (in milliseconds) using `time()`
```rust
time();
// 1761485668922.0
```

### plotting
when compiled with the `plot` feature (`cargo r --features plot`), you can use the plot function to plot a graph and output that plot to a png file
```rust
let delays = pass() ^ delay(0.01) ^ delay(0.02) ^ delay(0.03) ^ delay(0.04) ^ delay(0.05);
let g = saw_hz(4).phase(0) >> delays;
let seconds = 1;
let sr = 44100;
let ymin = -1;
let ymax = 1;
plot(g.clone(), seconds, sr, ymin, ymax); // you should find a file named plot.png
                                          // in your working directory

// arguments other than the graph can be omitted
// (with something that evaluates to none, like `_`)
// and those same defaults will be used
plot(g.clone(), _, _, _, _); // this is identical to the previous call

// you can provide a hue offset value (in 0..1 range)
plot(g.clone(), _, _, _, _, 0.5);

// you can specify the output file path, and its width and height
let w = 1000; // in pixels
let h = 500;
plot(g.clone(), _, _, _, _, _, "plots/extra_special_plot", w, h);
// (path has to be existing)
```

### sleep and panic
you can freeze or crash the app if you'd like. no judgement here
```rust
let duration = 2.5;
sleep(duration); // sleep for 2.5 seconds

panic(); // cause the app to crash
```

## building

- install rust: https://www.rust-lang.org/tools/install
- on linux you need `libjack-dev` and `libasound2-dev` (`jack-devel` and `alsa-lib-devel` on void)
- clone lapis
```
git clone https://codeberg.org/tomara-x/lapis.git
```
- build it
```
cd lapis
cargo run --release
```

## init file
the contents of the file `init.rs` (in the working directory) will be evaluated at startup

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

