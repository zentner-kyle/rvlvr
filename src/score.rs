use super::computed_distributions::{ComputedDistributions};
use super::operator::{Operator};

fn portion_correct_score(computed: usize, output: usize, dists: &[ComputedDistributions], targets: &[Vec<usize>]) -> f32 {
    let mut num_wrong = 0;
    for (d, t) in dists.iter().zip(targets.iter()) {
        let (v, _) = d.read_likely(computed);
        if t[output] != v {
            num_wrong += 1;
        }
    }
    let denom = f64::max(1.0, dists.len() as f64);
    1.0 - ((num_wrong as f64) / denom) as f32
}

fn total_complexity(computed: usize, operators: &[Operator]) -> usize {
    let mut total = 1;
    for dep in operators[computed].dependents().iter() {
        if let &Some(x) = dep {
            total += total_complexity(x, operators);
        }
    }
    total
}

fn infinite_to_1(x: f32) -> f32 {
    x / (1.0 + x)
}

fn complexity_score(computed: usize, operators: &[Operator]) -> f32 {
    1.0 - infinite_to_1(total_complexity(computed, operators) as f32)
}

fn log_mse_score(computed: usize, output: usize, dists: &[ComputedDistributions], targets: &[Vec<usize>]) -> f32 {
    let mut error = 0.0;
    for (d, t) in dists.iter().zip(targets.iter()) {
        for (v, pv) in d.read(computed).iter().enumerate() {
            let err = t[output] as i32 - v as i32;
            error += pv * pv * (err * err) as f32;
        }
    }
    1.0 - infinite_to_1(error)
}

fn combine_scores(old_score: f32, new_score: f32) -> f32 {
    f32::max(old_score, new_score)
}

pub fn compute_score_for_output(dists: &[ComputedDistributions], computed: usize, output: usize, targets: &[Vec<usize>], operators: &[Operator]) -> f32 {
    10.0 * portion_correct_score(computed, output, dists, targets) +
        5.0 * log_mse_score(computed, output, dists, targets) +
        complexity_score(computed, operators)
}

fn propagate_score(operators: &[Operator], scores: &mut [f32], i: usize, score: f32) {
    scores[i] = combine_scores(scores[i], score);
    for dep in operators[i].dependents().iter() {
        if let &Some(x) = dep {
            propagate_score(operators, scores, x, score);
        }
    }
}

pub fn score_values(dists: &[ComputedDistributions], i: usize, operators: &[Operator], scores: &mut [f32], targets: &[Vec<usize>]) -> (f32, usize) {
    let mut best_score = -1e9;
    let mut output = 0;
    for output_idx in 0..targets[0].len() {
        let score = compute_score_for_output(dists, i, output_idx, targets, operators);
        if score > best_score {
            best_score = score;
            output = output_idx;
        }
    }
    propagate_score(operators, scores, i, best_score);
    (best_score, output)
}

#[cfg(test)]
#[test]
fn it_scores_portions_correct() {
    assert_eq!(portion_correct_score(0, 0, &[], &[]), 1.0);
    let mut dists = ComputedDistributions::new(3, 2);
    dists.set_values(0, &[1, 2]);
    let d = &[dists.clone(), dists.clone()];
    assert_eq!(portion_correct_score(0, 0, &[dists.clone()], &[vec![1]]), 1.0);
    assert_eq!(portion_correct_score(0, 0, d, &[vec![1, 2], vec![1, 2]]), 1.0);
    assert_eq!(portion_correct_score(1, 1, d, &[vec![1, 2], vec![1, 2]]), 1.0);
    assert_eq!(portion_correct_score(0, 0, d, &[vec![1, 2], vec![0, 2]]), 0.5);
}
