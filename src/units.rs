use fundsp::hacker32::*;

/// multijoin and multisplit defined in:
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
