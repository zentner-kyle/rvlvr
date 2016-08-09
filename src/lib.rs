//! # A Genetic Programming System
//! rvlvr will be a genetic programming system. It efficiently implements DAG GP using dense
//! arrays.

extern crate rand;

/// An operator in a generated program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Initial,
    Value(i32),
    Equality(usize, usize),
    Increment(usize),
    Ambiguity(usize),
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
    Ite(usize, usize, usize),
}

fn operator_dependents(op: Operator) -> [Option<usize>; 3] {
    match op {
        Operator::Initial | Operator::Value(_) | Operator::Ambiguity(_) => [None, None, None],
        Operator::Increment(x) | Operator::Not(x) => [Some(x), None, None],
        Operator::Equality(x, y) | Operator::And(x, y) | Operator::Or(x, y) => [Some(x), Some(y), None],
        Operator::Ite(x, y, z) => [Some(x), Some(y), Some(z)],
    }
}

fn rand_idx<R>(rand_gen: &mut R, past_end: usize) -> usize where R: rand::Rng {
    rand_gen.next_u64() as usize % past_end
}

fn random_operator<R>(rand_gen: &mut R, output_idx: usize) -> Operator where R: rand::Rng {
    let total = 64;
    let mut op_idx = rand_gen.next_u32() % total;
    if false {
        panic!("no possible");
    } else if op_idx < 1 {
        Operator::Ambiguity(0)
    } else if op_idx < 2 {
        Operator::Value(0i32)
    } else if op_idx < 3 {
        Operator::Value(1i32)
    } else if op_idx < 4 {
        Operator::Value(2i32)
    } else if op_idx < 9 {
        Operator::Equality(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx))
    } else if op_idx < 19 {
        Operator::Increment(rand_idx(rand_gen, output_idx))
    } else if op_idx < 29 {
        Operator::And(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx))
    } else if op_idx < 39 {
        Operator::Or(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx))
    } else if op_idx < 49 {
        Operator::Not(rand_idx(rand_gen, output_idx))
    } else if op_idx < 64 {
        Operator::Ite(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx))
    } else {
        panic!("total needs to be updated above");
    }
}

fn random_operator_uniform<R>(rand_gen: &mut R, output_idx: usize) -> Operator where R: rand::Rng {
    let op_idx = rand_gen.next_u32() % 9;
    match op_idx {
        0 => Operator::Value(0i32),
        1 => Operator::Value(1i32),
        2 => Operator::Equality(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
        3 => Operator::Increment(rand_idx(rand_gen, output_idx)),
        4 => Operator::Ambiguity(0),
        5 => Operator::And(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
        6 => Operator::Or(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
        7 => Operator::Not(rand_idx(rand_gen, output_idx)),
        8 => Operator::Ite(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
        9 => Operator::Value(2i32),
        _ => panic!("herp"),
    }
}


pub fn do_operator<R>(trace: &mut [i32], operator: Operator, rand_gen: &mut R) -> i32 where R: rand::Rng {
    match operator {
        Operator::Initial => panic!("cannot perform Initial operator"),
        Operator::Value(i) => i,
        Operator::Equality(x, y) => if trace[x] == trace[y] { 1 } else { 0 },
        Operator::Increment(x) => trace[x] + 1,
        Operator::Ambiguity(_) => (rand_gen.next_u32() & 0x1) as i32,
        Operator::And(x, y) => if trace[x] != 0 && trace[y] != 0 { 1 } else { 0 },
        Operator::Or(x, y) => if trace[x] != 0 || trace[y] != 0 { 1 } else { 0 },
        Operator::Not(x) => if trace[x] == 0 { 1 } else { 0 },
        Operator::Ite(x, y, z) => if trace[x] == 0 { trace[y] } else { trace[z] },
    }
}

fn update_operator(operator: Operator, update: &[usize]) -> Operator {
    match operator {
        Operator::Initial => panic!("cannot update Initial operator"),
        Operator::Value(i) => Operator::Value(i),
        Operator::Equality(x, y) => Operator::Equality(update[x], update[y]),
        Operator::Increment(x) => Operator::Increment(update[x]),
        Operator::Ambiguity(i) => Operator::Ambiguity(i),
        Operator::And(x, y) => Operator::And(update[x], update[y]),
        Operator::Or(x, y) => Operator::Or(update[x], update[y]),
        Operator::Not(x) => Operator::Not(update[x]),
        Operator::Ite(x, y, z) => Operator::Ite(update[x], update[y], update[z]),
    }
}

pub fn portion_correct_score(computed: usize, variable: usize, traces: &[Vec<i32>], targets: &[&[i32]]) -> f32 {
    let mut num_wrong = 0;
    for (trace, target) in traces.iter().zip(targets.iter()) {
        if target[variable] != trace[computed] {
            num_wrong += 1;
        }
    }
    let denom = f64::max(1.0, traces.len() as f64);
    1.0 - ((num_wrong as f64) / denom) as f32
}

pub fn total_complexity(computed: usize, operators: &[Operator]) -> usize {
    let mut total = 1;
    for dep in operator_dependents(operators[computed]).iter() {
        if let &Some(x) = dep {
            total += total_complexity(x, operators);
        }
    }
    total
}

pub fn infinite_to_1(x: f32) -> f32 {
    x / (1.0 + x)
}

pub fn complexity_score(computed: usize, operators: &[Operator]) -> f32 {
    1.0 - infinite_to_1(total_complexity(computed, operators) as f32)
}

pub fn log_mse_score(computed: usize, variable: usize, traces: &[Vec<i32>], targets: &[&[i32]]) -> f32 {
    let mut error = 0.0;
    for (trace, target) in traces.iter().zip(targets.iter()) {
        let err = target[variable] - trace[computed];
        error += (err * err) as f32;
    }
    1.0 - infinite_to_1(error)
}

fn combine_scores(old_score: f32, new_score: f32) -> f32 {
    f32::max(old_score, new_score)
}

fn compute_score_for_variable(traces: &[Vec<i32>], i: usize, variable: usize, targets: &[&[i32]], operators: &[Operator]) -> f32 {
    10.0 * portion_correct_score(i, variable, traces, targets) + 5.0 * log_mse_score(i, variable, traces, targets) + complexity_score(i, operators)
}

fn propagate_score(operators: &[Operator], scores: &mut [f32], i: usize, score: f32) {
    scores[i] = combine_scores(scores[i], score);
    for dep in operator_dependents(operators[i]).iter() {
        if let &Some(x) = dep {
            scores[x] = combine_scores(scores[x], score);
        }
    }
}

fn score_values(traces: &[Vec<i32>], i: usize, operators: &[Operator], scores: &mut [f32], targets: &[&[i32]]) -> (f32, usize) {
    let mut best_score = -1e9;
    let mut variable = 0;
    for variable_idx in 0..targets[0].len() {
        let score = compute_score_for_variable(traces, i, variable_idx, targets, operators);
        if score > best_score {
            best_score = score;
            variable = variable_idx;
        }
    }
    propagate_score(operators, scores, i, best_score);
    (best_score, variable)
}

fn print_program(operators: &[Operator], i: usize) {
    match operators[i] {
        Operator::Initial => {
            print!("Initial({})", i);
        },
        Operator::Value(i) => {
            print!("Value({})", i);
        },
        Operator::Ambiguity(i) => {
            print!("Ambiguity({})", i);
        },
        Operator::Increment(x) => {
            print!("Increment(");
            print_program(operators, x);
            print!(")");
        },
        Operator::Not(x) => {
            print!("Not(");
            print_program(operators, x);
            print!(")");
        },
        Operator::Equality(x, y) => {
            print!("Equality(");
            print_program(operators, x);
            print!(", ");
            print_program(operators, y);
            print!(")");
        },
        Operator::And(x, y) => {
            print!("And(");
            print_program(operators, x);
            print!(", ");
            print_program(operators, y);
            print!(")");
        },
        Operator::Or(x, y) => {
            print!("Or(");
            print_program(operators, x);
            print!(", ");
            print_program(operators, y);
            print!(")");
        },
        Operator::Ite(x, y, z) => {
            print!("Ite(");
            print_program(operators, x);
            print!(", ");
            print_program(operators, y);
            print!(", ");
            print_program(operators, z);
            print!(")");
        },
    }
}

fn pretty_print_program(operators: &[Operator], i: usize) {
    match operators[i] {
        Operator::Initial => {
            print!("input[{}]", i);
        },
        Operator::Value(i) => {
            print!("{}", i);
        },
        Operator::Ambiguity(i) => {
            print!("ambiguous({})", i);
        },
        Operator::Increment(x) => {
            print!("1 + (");
            pretty_print_program(operators, x);
            print!(")");
        },
        Operator::Not(x) => {
            print!("!(");
            pretty_print_program(operators, x);
            print!(")");
        },
        Operator::Equality(x, y) => {
            print!("(");
            pretty_print_program(operators, x);
            print!(") == (");
            pretty_print_program(operators, y);
            print!(")");
        },
        Operator::And(x, y) => {
            print!("(");
            pretty_print_program(operators, x);
            print!(") && (");
            pretty_print_program(operators, y);
            print!(")");
        },
        Operator::Or(x, y) => {
            print!("(");
            pretty_print_program(operators, x);
            print!(") || (");
            pretty_print_program(operators, y);
            print!(")");
        },
        Operator::Ite(x, y, z) => {
            print!("if (");
            pretty_print_program(operators, x);
            print!(") {{ ");
            pretty_print_program(operators, y);
            print!("}} else {{");
            pretty_print_program(operators, z);
            print!("}}");
        },
    }
}


/// Finds transition functions for all variables.
///
/// The evolve algorithm works like this:
///
/// * Allocate the traces, operator, and update arrays.
/// * Populate the input values at the beginning of the result array.
/// * Populate the corresponding operator slots with the Init operator.
/// * Repeatedly:
///  * Fill all of operator and result arrays using random operators with random inputs draw from earlier in the result array.
///  * Prune the operator array, using some grading function. All operators that pass have new indices recorded in the update array.
///  * Move all traces and operators based on the update array.
///  * Check if done.
pub fn evolve(samples: &[&[&[i32]]], max_values: &[i32], size: usize, max_generations: usize) {
    let mut traces = Vec::with_capacity(samples.len());
    let mut targets = Vec::with_capacity(samples.len());
    let mut operators: Vec<Operator> = std::iter::repeat(Operator::Initial).take(size).collect();
    let mut scores: Vec<f32> = std::iter::repeat(0.0).take(size).collect();
    //let mut filtered_scores = scores.clone();
    let mut update: Vec<usize> = std::iter::repeat(0usize).take(size).collect();
    let input_size = samples[0][0].len();
    let mut pop_size = input_size;
    for sample in samples.iter() {
        for values in sample.windows(2) {
            let start = values[0];
            let end = values[1];
            let mut trace = Vec::with_capacity(size);
            trace.extend_from_slice(start);
            trace.extend(std::iter::repeat(0i32).take(size - input_size));
            traces.push(trace);
            targets.push(end);
        }
    }
    use rand::{Rng, SeedableRng};
    //let mut rand_gen = rand::weak_rng();
    //let mut rand_gen = rand::thread_rng();
    let mut rand_gen = rand::XorShiftRng::from_seed([0xde, 0xad, 0xbe, 0xef]);
    let mut generation = 0;
    loop {
        // Generate random new operators.
        for i in pop_size..size {
            //filtered_scores[i] = 0.0;
            let mut operator = random_operator(&mut rand_gen, i);
            operators[i] = operator;
            for trace in traces.iter_mut() {
                let result = do_operator(trace, operator, &mut rand_gen);
                trace[i] = result;
            }
        }
        let mut avg_score: f32 = 0.0;
        let filter_alpha = 0.5f32;
        let num_scores = size - input_size;
        for i in (input_size..size).rev() {
            score_values(&traces, i, &operators, &mut scores, &targets);
            //if filtered_scores[i] == 0.0 {
                //filtered_scores[i] = scores[i];
            //} else {
                //filtered_scores[i] = scores[i] * (1.0 - filter_alpha) + filtered_scores[i] * filter_alpha;
                //avg_score += scores[i] / num_scores as f32;
            //}
        }
        //println!("avg_score = {}", avg_score);
        let mut next_out = input_size;
        for i in input_size..size {
            if scores[i] > avg_score || rand_gen.next_f32() > 0.25 {
                update[i] = next_out;
                operators[next_out] = update_operator(operators[i], &update);
                //filtered_scores[next_out] = filtered_scores[i];
                next_out += 1;
            } else {
                update[i] = 0;
            }
        }
        pop_size = next_out;
        generation += 1;
        if generation > max_generations {
            for variable in 0..input_size {
                let mut best_score = 0.0;
                let mut best_computed = 0;
                for i in 0..size {
                    let score = compute_score_for_variable(&traces, i, variable, &targets, &operators);
                    if score > best_score {
                        best_score = score;
                        best_computed = i;
                    }
                }
                println!("best program (scores {}) for {}:", best_score, variable);
                pretty_print_program(&operators, best_computed);
                println!("");
                for (trace, target) in traces.iter().zip(targets.iter()) {
                    println!("{} vs {}", target[variable], trace[best_computed]);
                }
            }
            break;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use rand::{Rng, SeedableRng};

    #[test]
    fn it_scores_portions_correct() {
        assert_eq!(portion_correct_score(0, 0, &[], &[]), 1.0);
        assert_eq!(portion_correct_score(0, 0, &[vec![1]], &[&[1]]), 1.0);
        assert_eq!(portion_correct_score(0, 0, &[vec![1]], &[&[0]]), 0.0);
        assert_eq!(portion_correct_score(0, 0, &[vec![1], vec![2]], &[&[1], &[2]]), 1.0);
        assert_eq!(portion_correct_score(0, 0, &[vec![1], vec![0]], &[&[1], &[2]]), 0.5);
    }

    #[test]
    fn it_performs_operations() {
        let mut rand_gen = rand::XorShiftRng::from_seed([0xde, 0xad, 0xbe, 0xef]);
        let mut trace = &mut [0, 0, 1, 2];
        assert_eq!(do_operator(trace, Operator::Increment(2), &mut rand_gen), 2);
        assert_eq!(do_operator(trace, Operator::Value(10), &mut rand_gen), 10);
        assert_eq!(do_operator(trace, Operator::Not(0), &mut rand_gen), 1);
        assert_eq!(do_operator(trace, Operator::Not(2), &mut rand_gen), 0);
        assert_eq!(do_operator(trace, Operator::Not(3), &mut rand_gen), 0);
        assert_eq!(do_operator(trace, Operator::Not(0), &mut rand_gen), 1);
        assert_eq!(do_operator(trace, Operator::Equality(0, 1), &mut rand_gen), 1);
    }

    #[test]
    fn it_evolves_122() {
        // variables are:
        // winner
        // player
        // counter
        // There are only two possible games. ;)
        evolve(&[&[&[2, 0, 0],
                   &[2, 1, 1],
                   &[2, 0, 2],
                   &[0, 1, 2],],
                 &[&[2, 0, 0],
                   &[2, 1, 2],
                   &[1, 0, 2]]],
               &[2, 1, 2],
               //16,
               1024,
               //2);
               //1);
               1024);
               //10);
    }
}
