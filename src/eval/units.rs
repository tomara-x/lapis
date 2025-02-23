use crossbeam_channel::{Receiver, Sender};
use fundsp::hacker32::*;

/// multijoin, multisplit, and reverse defined in:
/// https://github.com/SamiPerttu/fundsp/blob/master/src/audionode.rs
/// with small changes to make them work as `AudioUnit`s instead
#[derive(Clone)]
pub struct MultiSplitUnit {
    inputs: usize,
    outputs: usize,
}
impl MultiSplitUnit {
    pub fn new(inputs: usize, splits: usize) -> Self {
        let outputs = inputs * splits;
        MultiSplitUnit { inputs, outputs }
    }
}
impl AudioUnit for MultiSplitUnit {
    fn reset(&mut self) {}

    fn set_sample_rate(&mut self, _sample_rate: f64) {}

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        for i in 0..self.outputs {
            output[i] = input[i % self.inputs];
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..self.outputs {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(channel % self.inputs, i));
            }
        }
    }

    fn inputs(&self) -> usize {
        self.inputs
    }

    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Split.route(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 138;
        ID
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[derive(Clone)]
pub struct MultiJoinUnit {
    outputs: usize,
    branches: usize,
}
impl MultiJoinUnit {
    pub fn new(outputs: usize, branches: usize) -> Self {
        MultiJoinUnit { outputs, branches }
    }
}
impl AudioUnit for MultiJoinUnit {
    fn reset(&mut self) {}

    fn set_sample_rate(&mut self, _sample_rate: f64) {}

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        for j in 0..self.outputs {
            let mut out = input[j];
            for i in 1..self.branches {
                out += input[j + i * self.outputs];
            }
            output[j] = out / self.branches as f32;
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let z = 1.0 / self.branches as f32;
        for channel in 0..self.outputs {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(channel, i) * z);
            }
        }
        for channel in self.outputs..self.outputs * self.branches {
            for i in 0..simd_items(size) {
                output.add(channel % self.outputs, i, input.at(channel, i) * z);
            }
        }
    }

    fn inputs(&self) -> usize {
        self.outputs * self.branches
    }

    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Join.route(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 139;
        ID
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

#[derive(Clone)]
pub struct ReverseUnit {
    n: usize,
}
impl ReverseUnit {
    pub fn new(n: usize) -> Self {
        ReverseUnit { n }
    }
}
impl AudioUnit for ReverseUnit {
    fn reset(&mut self) {}

    fn set_sample_rate(&mut self, _sample_rate: f64) {}

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        for i in 0..self.n {
            output[i] = input[self.n - 1 - i];
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..self.n {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(self.n - 1 - channel, i));
            }
        }
    }

    fn inputs(&self) -> usize {
        self.n
    }

    fn outputs(&self) -> usize {
        self.n
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Reverse.route(input, self.n)
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 145;
        ID
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

/// mic input node
/// - output 0: left
/// - output 1: right
#[derive(Clone)]
pub struct InputNode {
    lr: Receiver<f32>,
    rr: Receiver<f32>,
}
impl InputNode {
    pub fn new(lr: Receiver<f32>, rr: Receiver<f32>) -> Self {
        InputNode { lr, rr }
    }
}
impl AudioNode for InputNode {
    const ID: u64 = 1117;
    type Inputs = U0;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let l = self.lr.try_recv().unwrap_or(0.);
        let r = self.rr.try_recv().unwrap_or(0.);
        [l, r].into()
    }
}

/// send samples to crossbeam channel
/// - input 0: input
/// - output 0: input passed through
#[derive(Clone)]
pub struct BuffIn {
    s: Sender<f32>,
}
impl BuffIn {
    pub fn new(s: Sender<f32>) -> Self {
        BuffIn { s }
    }
}
impl AudioNode for BuffIn {
    const ID: u64 = 1123;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let _ = self.s.try_send(input[0]);
        [input[0]].into()
    }
}

/// receive smaples from crossbeam channel
/// - output 0: output
#[derive(Clone)]
pub struct BuffOut {
    r: Receiver<f32>,
}
impl BuffOut {
    pub fn new(r: Receiver<f32>) -> Self {
        BuffOut { r }
    }
}
impl AudioNode for BuffOut {
    const ID: u64 = 1124;
    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        [self.r.try_recv().unwrap_or_default()].into()
    }
}
