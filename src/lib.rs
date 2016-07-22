extern crate rand;

/// rvlvr will be a genetic programming system. It efficiently implements DAG GP using dense
/// arrays.

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

fn rand_idx<R>(rand_gen: &mut R, past_end: usize) -> usize where R: rand::Rng {
    rand_gen.next_u64() as usize % past_end
}

fn random_operator<R>(rand_gen: &mut R, output_idx: usize) -> Operator where R: rand::Rng {
    let total = 64;
    let mut op_idx = rand_gen.next_u32() % total;
    if op_idx < 1 {
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


fn do_operator<R>(trace: &mut [i32], operator: Operator, rand_gen: &mut R) -> i32 where R: rand::Rng {
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

fn combine_scores(old_score: f32, new_score: f32) -> f32 {
    f32::max(old_score, new_score)
}

fn score_values(traces: &[Vec<i32>], i: usize, operators: &[Operator], scores: &mut [f32], targets: &[&[i32]]) -> (f32, usize) {
    let operator = operators[i];
    let mut best_score = -1e9;
    let mut variable = 0;
    for variable_idx in 0..targets[0].len() {
        //println!("checking variable {}", variable_idx);
        let mut score = 10.0;
        for (trace, target) in traces.iter().zip(targets.iter()) {
            //println!("target = {}", target[variable_idx]);
            //println!("trace = {}", trace[i]);
            let err = target[variable_idx] - trace[i];
            println!("err = {}", err);
            score -= (err * err) as f32;
        }
        if score > best_score {
            best_score = score;
            variable = variable_idx;
        }
    }
    if best_score == 10.0 {
        println!("predicted {}", variable);
        print_program(operators, i);
        println!("");
    }
    scores[i] = combine_scores(scores[i], best_score);
    match operator {
        Operator::Initial => {},
        Operator::Value(_) => {},
        Operator::Ambiguity(_) => {},
        Operator::Increment(x) | Operator::Not(x) => {
            scores[x] = combine_scores(scores[x], scores[i]);
        },
        Operator::Equality(x, y) | Operator::And(x, y) | Operator::Or(x, y) => {
            scores[x] = combine_scores(scores[x], scores[i]);
            scores[y] = combine_scores(scores[y], scores[i]);
        },
        Operator::Ite(x, y, z) => {
            scores[x] = combine_scores(scores[x], scores[i]);
            scores[y] = combine_scores(scores[y], scores[i]);
            scores[z] = combine_scores(scores[z], scores[i]);
        },
    }
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
    let mut filtered_scores = scores.clone();
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
            filtered_scores[i] = 0.0;
            let mut operator = random_operator(&mut rand_gen, i);
            operators[i] = operator;
            for trace in traces.iter_mut() {
                let result = do_operator(trace, operator, &mut rand_gen);
                trace[i] = result;
            }
        }
        let mut avg_score: f32 = 0.0;
        let filter_alpha = 0.99f32;
        let num_scores = size - input_size;
        for i in (input_size..size).rev() {
            score_values(&traces, i, &operators, &mut scores, &targets);
            filtered_scores[i] = scores[i] * (1.0 - filter_alpha) + filtered_scores[i] * filter_alpha;
            avg_score += scores[i] / num_scores as f32;
        }
        //println!("avg_score = {}", avg_score);
        let mut next_out = input_size;
        for i in input_size..size {
            if scores[i] > avg_score || rand_gen.next_f32() > 0.25 {
                update[i] = next_out;
                operators[next_out] = update_operator(operators[i], &update);
                filtered_scores[next_out] = filtered_scores[i];
                next_out += 1;
            } else {
                update[i] = 0;
            }
        }
        pop_size = next_out;
        generation += 1;
        if generation > max_generations {
            //println!("operators: {:?}", operators);
            //println!("scores: {:?}", scores);
            //println!("filtered_scores: {:?}", filtered_scores);
            let mut score_idx_vec: Vec<(usize, f32)> = filtered_scores.iter().cloned().enumerate().collect();
            score_idx_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Less));
            for variable in 0..input_size {
                for &(i, filtered_score) in score_idx_vec.iter() {
                    let (score, varb) = score_values(&traces, i, &operators, &mut scores, &targets);
                    if variable == varb {
                        println!("best program (scores {}, {}) for {}:", score, filtered_score, variable);
                        print_program(&operators, i);
                        println!("");
                        break;
                    }
                }
            }
            break;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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
               16,
               //8 * 1024,
               2);
    }
}
