use rand;

/// An operator in a generated program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Initial,
    Value(usize),
    Equality(usize, usize),
    Increment(usize),
    Ambiguity(usize),
    And(usize, usize),
    Or(usize, usize),
    Not(usize),
    Ite(usize, usize, usize),
}

impl Operator {
    pub fn dependents(&self) -> [Option<usize>; 3] {
        match *self {
            Operator::Initial | Operator::Value(_) | Operator::Ambiguity(_) => [None, None, None],
            Operator::Increment(x) | Operator::Not(x) => [Some(x), None, None],
            Operator::Equality(x, y) | Operator::And(x, y) | Operator::Or(x, y) => [Some(x), Some(y), None],
            Operator::Ite(x, y, z) => [Some(x), Some(y), Some(z)],
        }
    }

    pub fn new_rand<R>(rand_gen: &mut R, output_idx: usize) -> Operator where R: rand::Rng {
        let total = 64;
        let mut op_idx = rand_gen.next_u32() % total;
        if false {
            panic!("no possible");
        } else if op_idx < 1 {
            Operator::Ambiguity(0)
        } else if op_idx < 2 {
            Operator::Value(0)
        } else if op_idx < 3 {
            Operator::Value(1)
        } else if op_idx < 4 {
            Operator::Value(2)
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

    pub fn new_rand_uniform<R>(rand_gen: &mut R, output_idx: usize) -> Operator where R: rand::Rng {
        let op_idx = rand_gen.next_u32() % 9;
        match op_idx {
            0 => Operator::Value(0),
            1 => Operator::Value(1),
            2 => Operator::Equality(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
            3 => Operator::Increment(rand_idx(rand_gen, output_idx)),
            4 => Operator::Ambiguity(0),
            5 => Operator::And(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
            6 => Operator::Or(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
            7 => Operator::Not(rand_idx(rand_gen, output_idx)),
            8 => Operator::Ite(rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx), rand_idx(rand_gen, output_idx)),
            9 => Operator::Value(2),
            _ => panic!("op_idx divisor above needs updating"),
        }
    }

    fn exec<R>(&self, args: [Option<usize>; 3], rand_gen: &mut R) -> usize where R: rand::Rng {
        0
        //Operator::Initial => panic!("cannot perform Initial operator"),
        //Operator::Value(i) => i,
        //Operator::Equality(x, y) => if trace[x] == trace[y] { 1 } else { 0 },
        //Operator::Increment(x) => trace[x] + 1,
        //Operator::Ambiguity(_) => (rand_gen.next_u32() & 0x1) as i32,
        //Operator::And(x, y) => if trace[x] != 0 && trace[y] != 0 { 1 } else { 0 },
        //Operator::Or(x, y) => if trace[x] != 0 || trace[y] != 0 { 1 } else { 0 },
        //Operator::Not(x) => if trace[x] == 0 { 1 } else { 0 },
        //Operator::Ite(x, y, z) => if trace[x] == 0 { trace[y] } else { trace[z] },
    }

    pub fn run(&self, target: usize, dists: &mut super::computed_distributions::ComputedDistributions) {
        match *self {
            Operator::Initial => panic!("cannot run Initial operator"),
            Operator::Value(i) => dists.compute_at_0(target, || i),
            Operator::Equality(x, y) => dists.compute_at_2(target, (x, y),
                |x, y| {
                    if x == y { 1 } else { 0 }
                }),
            Operator::And(x, y) => dists.compute_at_2(target, (x, y),
                |x, y| {
                    if x != 0 && y != 0 { 1 } else { 0 }
                }),
            Operator::Or(x, y) => dists.compute_at_2(target, (x, y),
                |x, y| {
                    if x != 0 || y != 0 { 1 } else { 0 }
                }),
            Operator::Increment(x) => dists.compute_at_1(target, x, |x| x + 1),
            Operator::Not(x) => dists.compute_at_1(target, x, |x| if x != 0 { 0 } else { 1 }),
            Operator::Ite(x, y, z) => dists.compute_at_3(target, (x, y, z),
                |x, y, z| {
                    if x != 0 {
                        y
                    } else {
                        z
                    }
                }),
            Operator::Ambiguity(_) => dists.compute_at_0_prob(target,
                |out| {
                    out[0] = 0.5;
                    out[1] = 0.5;
                }),
        }
    }

    pub fn relocate(&self, relocations: &[Option<usize>]) -> Self {
        let reason = "All dependent operators should have been relocated.";
        match *self {
            Operator::Initial => panic!("cannot relocate Initial operator"),
            Operator::Value(i) => Operator::Value(i),
            Operator::Ambiguity(i) => Operator::Ambiguity(i),
            Operator::Increment(x) => Operator::Increment(relocations[x].expect(reason)),
            Operator::Not(x) => Operator::Not(relocations[x].expect(reason)),
            Operator::Equality(x, y) =>
                Operator::Equality(relocations[x].expect(reason), relocations[y].expect(reason)),
            Operator::And(x, y) =>
                Operator::And(relocations[x].expect(reason), relocations[y].expect(reason)),
            Operator::Or(x, y) =>
                Operator::Or(relocations[x].expect(reason), relocations[y].expect(reason)),
            Operator::Ite(x, y, z) =>
                Operator::Ite(relocations[x].expect(reason),
                              relocations[y].expect(reason),
                              relocations[z].expect(reason)),
        }
    }
}

fn rand_idx<R>(rand_gen: &mut R, past_end: usize) -> usize where R: rand::Rng {
    rand_gen.next_u64() as usize % past_end
}

pub fn print_program(operators: &[Operator], i: usize) {
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

pub fn pretty_print_program(operators: &[Operator], i: usize) {
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
            print!(" }} else {{ ");
            pretty_print_program(operators, z);
            print!(" }}");
        },
    }
}
