///! Stores an array of probability distributions over a finite, non-negative integer domain.
///! Each such distribution is called a "computed distribution." These distributions correspond to
///! the computed probability distributions for the corresponding "computed."
use std;

#[derive(Clone, Debug)]
pub struct ComputedDistributions {
    size: usize,
    values: Vec<f32>,
}

impl ComputedDistributions {
    pub fn new(size: usize, count: usize) -> Self {
        ComputedDistributions {
            size: size,
            values: std::iter::repeat(0.0).take((size + 1) * count).collect(),
        }
    }

    pub fn set_values(&mut self, offset: usize, values: &[usize]) {
        for (i, val) in values.iter().enumerate() {
            let mut slice = self.read_mut(offset + i);
            for (j, v) in slice.iter_mut().enumerate() {
                if *val == j {
                    *v = 1.0;
                } else {
                    *v = 0.0;
                }
            }
        }
    }

    fn computed_idx(&self, computed: usize) -> usize {
        computed * (self.size + 1)
    }

    pub fn store(&mut self, computed: usize, distribution: &[f32]) {
        assert_eq!(distribution.len(), self.size + 1);
        let idx = self.computed_idx(computed);
        copy_into_slice(&mut self.values, idx, distribution);
    }

    pub fn read(&self, computed: usize) -> &[f32] {
        self.values.split_at(self.computed_idx(computed)).1.split_at(self.size + 1).0
    }

    pub fn read_likely(&self, computed: usize) -> (usize, f32) {
        let mut best = 0;
        let mut best_prob = 0.0;
        for (v, &pv) in self.read(computed).iter().enumerate() {
            if pv > best_prob {
                best_prob = pv;
                best = v;
            }
        }
        (best, best_prob)
    }

    pub fn read_mut(&mut self, computed: usize) -> &mut[f32] {
        let idx = self.computed_idx(computed);
        let size = self.size;
        self.values.split_at_mut(idx).1.split_at_mut(size + 1).0
    }

    pub fn relocate(&mut self, relocations: &[Option<usize>]) {
        for (i, d) in relocations.iter().enumerate() {
            if let &Some(d) = d {
                // Is there a clean way to avoid this allocation?
                let dist = self.read(i).to_owned();
                self.store(d, &dist);
            }
        }
    }

    pub fn compute_at_3<F>(&mut self, target: usize, srcs: (usize, usize, usize), f: F)
        where F: Fn(usize, usize, usize) -> usize {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(srcs.0);
        let y_idx = self.computed_idx(srcs.1);
        let z_idx = self.computed_idx(srcs.2);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // If any input in undefined, the output is undefined.
        self.values[target_idx + self.size] += self.values[x_idx + self.size];
        self.values[target_idx + self.size] += self.values[y_idx + self.size];
        self.values[target_idx + self.size] += self.values[z_idx + self.size];
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size)).enumerate() {
            for (y, yi) in (y_idx..(y_idx + self.size)).enumerate() {
                for (z, zi) in (z_idx..(z_idx + self.size)).enumerate() {
                    let mut out = f(x, y, z);
                    // If out is too large, then it's not defined.
                    if out >= self.size {
                        out = self.size;
                    }
                    // Add the probability that all three computed have these values.
                    self.values[target_idx + out] += self.values[xi] * self.values[yi] * self.values[zi];
                }
            }
        }
    }

    pub fn compute_at_2<F>(&mut self, target: usize, srcs: (usize, usize), f: F)
        where F: Fn(usize, usize) -> usize {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(srcs.0);
        let y_idx = self.computed_idx(srcs.1);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // If any input in undefined, the output is undefined.
        self.values[target_idx + self.size] += self.values[x_idx + self.size];
        self.values[target_idx + self.size] += self.values[y_idx + self.size];
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size)).enumerate() {
            for (y, yi) in (y_idx..(y_idx + self.size)).enumerate() {
                let mut out = f(x, y);
                // If out is too large, then it's not defined.
                if out >= self.size {
                    out = self.size;
                }
                // Add the probability that all three computed have these values.
                self.values[target_idx + out] += self.values[xi] * self.values[yi];
            }
        }
    }

    pub fn compute_at_1<F>(&mut self, target: usize, src: usize, f: F)
        where F: Fn(usize) -> usize {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(src);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // If any input in undefined, the output is undefined.
        self.values[target_idx + self.size] += self.values[x_idx + self.size];
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size)).enumerate() {
            let mut out = f(x);
            // If out is too large, then it's not defined.
            if out >= self.size {
                out = self.size;
            }
            // Add the probability that all three computed have these values.
            self.values[target_idx + out] += self.values[xi];
        }
    }

    pub fn compute_at_0<F>(&mut self, target: usize, f: F)
        where F: Fn() -> usize {
        let target_idx = self.computed_idx(target);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        let mut out = f();
        // If out is too large, then it's not defined.
        if out >= self.size {
            out = self.size;
        }
        self.values[target_idx + out] = 1.0;
    }

    pub fn compute_at_0_prob<F>(&mut self, target: usize, f: F)
        where F: Fn(&mut[f32]) {
        let target_idx = self.computed_idx(target);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        f(self.read_mut(target));
    }

    pub fn compute_at_1_prob<F>(&mut self, target: usize, src: usize, f: F)
        where F: Fn(&mut[f32], usize, f32) {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(src);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size + 1)).enumerate() {
            let px = self.values[xi];
            f(self.read_mut(target), x, px);
        }
    }

    pub fn compute_at_2_prob<F>(&mut self, target: usize, srcs: (usize, usize), f: F)
        where F: Fn(&mut[f32], usize, f32, usize, f32) {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(srcs.0);
        let y_idx = self.computed_idx(srcs.1);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size + 1)).enumerate() {
            for (y, yi) in (y_idx..(y_idx + self.size + 1)).enumerate() {
                let px = self.values[xi];
                let py = self.values[yi];
                f(self.read_mut(target), x, px, y, py);
            }
        }
    }

    pub fn compute_at_3_prob<F>(&mut self, target: usize, srcs: (usize, usize, usize), f: F)
        where F: Fn(&mut[f32], usize, f32, usize, f32, usize, f32) {
        let target_idx = self.computed_idx(target);
        let x_idx = self.computed_idx(srcs.0);
        let y_idx = self.computed_idx(srcs.1);
        let z_idx = self.computed_idx(srcs.2);
        // Clear out all target values.
        for t in target_idx..(target_idx + self.size + 1) {
            self.values[t] = 0.0;
        }
        // For each combination of inputs, compute the output value.
        for (x, xi) in (x_idx..(x_idx + self.size + 1)).enumerate() {
            for (y, yi) in (y_idx..(y_idx + self.size + 1)).enumerate() {
                for (z, zi) in (z_idx..(z_idx + self.size + 1)).enumerate() {
                    let px = self.values[xi];
                    let py = self.values[yi];
                    let pz = self.values[zi];
                    f(self.read_mut(target), x, px, y, py, z, pz);
                }
            }
        }
    }
}

fn copy_into_slice<T>(dest: &mut [T], offset: usize, src: &[T]) where T: Clone {
    let dlen = dest.len();
    let slen = src.len();
    if offset + slen > dlen {
        panic!("Destination is not large enough");
    } else {
        for (s, d) in (offset..(offset + slen)).enumerate() {
            dest[d] = src[s].clone();
        }
    }
}
