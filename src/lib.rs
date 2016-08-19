//! # A Genetic Programming System
//! rvlvr will be a genetic programming system. It efficiently implements DAG GP using dense
//! arrays.

extern crate rand;

mod computed_distributions;
mod operator;
mod score;
mod evolver;
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
pub fn evolve(samples: &[&[&[usize]]], max_value: usize, size: usize, max_generations: usize) {
    let mut evolver = evolver::Evolver::new(samples, max_value, size);
    evolver.run_generations(max_generations);
    evolver.print_best();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_computes_probabilities() {
        let mut distributions = super::computed_distributions::ComputedDistributions::new(2, 4);
        distributions.store(0, &[0.5, 0.5, 0.0]);
        distributions.store(1, &[1.0, 0.0, 0.0]);
        distributions.store(2, &[0.0, 1.0, 0.0]);
        distributions.compute_at_3(3, (0, 1, 2), |x, y, z| {
            if x != 0 {
                y
            } else {
                z
            }
        });
        assert_eq!(distributions.read(3), &[0.5, 0.5, 0.0]);
    }

    #[test]
    fn it_computes_ite() {
        let mut distributions = super::computed_distributions::ComputedDistributions::new(2, 4);
        distributions.store(0, &[0.5, 0.5, 0.0]);
        distributions.store(1, &[1.0, 0.0, 0.0]);
        distributions.store(2, &[0.0, 1.0, 0.0]);
        super::operator::Operator::Ite(0, 1, 2).run(3, &mut distributions);
        assert_eq!(distributions.read(3), &[0.5, 0.5, 0.0]);
    }

    #[test]
    fn it_computes_not() {
        let mut distributions = super::computed_distributions::ComputedDistributions::new(2, 4);
        distributions.store(0, &[0.5, 0.5, 0.0]);
        distributions.store(1, &[1.0, 0.0, 0.0]);
        distributions.store(2, &[0.0, 1.0, 0.0]);
        super::operator::Operator::Not(0).run(3, &mut distributions);
        assert_eq!(distributions.read(3), &[0.5, 0.5, 0.0]);
        super::operator::Operator::Not(1).run(3, &mut distributions);
        assert_eq!(distributions.read(3), &[0.0, 1.0, 0.0]);
        super::operator::Operator::Not(2).run(3, &mut distributions);
        assert_eq!(distributions.read(3), &[1.0, 0.0, 0.0]);
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
               2,
               16,
               128);
    }
}
