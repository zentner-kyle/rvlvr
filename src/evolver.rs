use rand;
use rand::{Rng, SeedableRng};
use super::computed_distributions::{ComputedDistributions};
use super::operator::{Operator};
use super::score::{score_values, compute_score_for_output};

pub struct Evolver {
    computed: Vec<ComputedDistributions>,
    operators: Vec<Operator>,
    scores: Vec<f32>,
    relocations: Vec<Option<usize>>,
    targets: Vec<Vec<usize>>,
    max_value: usize,
    population_size: usize,
    generation: usize,
    rand_gen: rand::XorShiftRng,
    input_size: usize,
    done_count: usize,
}

impl Evolver {
    pub fn new(samples: &[&[&[usize]]], max_value: usize, population_size: usize) -> Self {
        let input_size = samples[0][0].len();
        use std::iter::{repeat};
        let size = population_size;
        let mut computed = Vec::with_capacity(samples.len());
        let mut targets = Vec::with_capacity(samples.len());
        for sample in samples.iter() {
            for values in sample.windows(2) {
                let start = values[0];
                let end = values[1];
                let mut dists = ComputedDistributions::new(max_value + 1, population_size);
                dists.set_values(0, start);
                computed.push(dists);
                targets.push(end.to_owned());
            }
        }
        let mut relocations: Vec<Option<usize>> = repeat(None).take(size).collect();
        for i in 0..input_size {
            relocations[i] = Some(i);
        }
        let rand_gen = rand::XorShiftRng::from_seed([0xde, 0xad, 0xbe, 0xef]);
        let operators = repeat(Operator::Initial).take(size).collect();
        let scores = repeat(0.0).take(size).collect();
        Evolver {
            computed: computed,
            operators: operators,
            scores: scores,
            relocations: relocations,
            targets: targets,
            max_value: max_value,
            population_size: population_size,
            generation: 0,
            rand_gen: rand_gen,
            input_size: input_size,
            done_count: input_size,
        }
    }

    pub fn populate(&mut self) {
        for i in self.done_count..self.population_size {
            self.operators[i] = Operator::new_rand(&mut self.rand_gen, i);
        }
    }

    pub fn evaluate(&mut self) {
        for i in self.done_count..self.population_size {
            for dists in self.computed.iter_mut() {
                self.operators[i].run(i, dists);
            }
        }
    }

    pub fn score(&mut self) {
        for i in self.input_size..self.population_size {
            self.scores[i] = 0.0;
            score_values(&self.computed, i, &self.operators, &mut self.scores, &self.targets);
        }
    }

    pub fn prune(&mut self) {
        let mut avg_score = 0.0;
        for i in 0..self.population_size {
            avg_score += self.scores[i] / self.population_size as f32;
        }
        let mut next_out = self.input_size;
        for i in self.input_size..self.population_size {
            if self.scores[i] >= avg_score {
                self.relocations[i] = Some(next_out);
                self.operators[next_out] = self.operators[i].relocate(&self.relocations);
                self.scores[next_out] = self.scores[i];

                next_out += 1;
            } else {
                self.relocations[i] = None;
            }
        }
        for dists in self.computed.iter_mut() {
            dists.relocate(&self.relocations);
        }
        self.done_count = next_out;
        self.generation += 1;
    }

    pub fn run_generations(&mut self, generations: usize) {
        for _ in 0..generations {
            self.populate();
            self.evaluate();
            self.score();
            self.prune();
        }
    }

    pub fn print_best(&self) {
        let output_size = self.targets[0].len();
        for output in 0..output_size {
            let mut best_score = 0.0;
            let mut best_computed = 0;
            for i in self.input_size..self.population_size {
                let score = compute_score_for_output(&self.computed, i, output, &self.targets, &self.operators);
                if score > best_score {
                    best_score = score;
                    best_computed = i;
                }
            }
            println!("best program (scores {}) for {}:", best_score, output);
            super::operator::pretty_print_program(&self.operators, best_computed);
            println!("");
            for (dist, target) in self.computed.iter().zip(self.targets.iter()) {
                let (pred, prob) = dist.read_likely(best_computed);
                println!("predicted {} with prob {} vs target {}", pred, prob, target[output]);
            }
        }
    }
}
