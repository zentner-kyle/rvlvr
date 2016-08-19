How should we handle evolving *random* programs?
================================================

Non-random programs are relatively easy to score, since they have defined
traces. However, random programs actually each have a distribution over a DAG
of values, any path along which is a valid trace.

One perspective is that there are two main problems.

First, we would like to prevent random programs from scoring unreasonably well,
because a purely random program can produce the correct output for any input,
and is extremely simple, but is not a useful program. For example, `arb()` may
score as well as `!input[1]` for predicting the next turn in many games (when
`input[1]` is the current turn). However, `input[1]` is the correct program.

Equivalently, we would like the conditions where random behavior are used to be
minimal. Essentially, we would like to find programs that explain as much
behavior as possible, and leave some of the behavior to unknown noise.

Secondly, we need to actually estimate how good the random program is
efficiently. Since the number of possible traces increases exponentially with
the number of random operators, this wouldn't be efficient to compute by
computing each trace.

I suspect that the current operator, `arb()`, is poorly suited to enforcing the
constraints we would like. I propose a new operator known as `ime(x, y, z)`
`(if x maybe y else z)`. Essentially, this operator acts like `ite(x, y, z)`,
except that with some probability `z` is returned even when `x` is true. The
primary drawback is that this adds another trinary operator. Furthermore, this
trinary operator could reasonably also have a prior associated with it. This
would require four fields, which would increase the size of the operator
itself. However, this is probably not a major concern. This implementation of
DAG GP does not seem to be constrained by memory usage.

The main benefit of `ime(x, y, z)` is that a more specific (rarely true) `x`
indicates a less random program. This provides an indirect method of
encouraging program non-randomness.

I think it's also worth mentioning that `arb()` is difficult to use for
selecting a value from an array at random. We will probably want some type of
generalized `mindex(x, y, z)` operator, may index `y` at a random index if `x`
is true, and otherwise indexes `y` at `z`. In comparison, achieving an
equivalent effect with `arb()` is extremely difficult.

What options do we have?

1. Evaluate random traces repeatedly to sample the probability distribution.
   This is extremely wasteful at the beginning of the distribution, especially
   when we can efficiently analytically compute the probability that a trace
   location has a particular value.
2. Analytically compute the probability that each trace location has each
   value, assuming that the distributions of all computed earlier in the trace
   are uncorrelated. This assumption is blantantly false, however it should
   only result in the distributions of late computed becoming too wide, not to
   narrow. Since the evolution objective rewards narrowing the distribution,
   this effect will hopefully be overcome by the evolution process.
   Alternatively, there is a risk that spreading effect will result in
   under-rewarding optimal members of the population. The cost of this method
   increases linearly with each possible value. As long as we are computing
   over small finite fields, this may be the most efficient method.  However,
   scaling it to larger domains may require also using the first method.
3. Each generation, randomly evaluate the random operator. Given enough
   generations, this should be equivalent to 1. However, it's not terribly
   efficient. It also implies that we must re-evaluate previous generations,
   instead of simply re-using their computed values.
4. Cap the number of random operators in use in each trace, and compute the
   2^cap results. Score based on the number correct versus incorrect. This has
   the advantage of working well regardless of the operators used, and directly
   prevents excessive randomness. However, this is very expensive for dealing
   with highly complex random behavior, and requires that the random operator
   has two possible results. It's not clear how this can be extended to
   `mindex(x, y, z)`.
5. Specially priviledge random operators at the end of a trace, by evaluating
   them multiple times and scoring based on the best produced value.


Given our constraints, I think that using the second method, possibly extended
with the first method in the future, is the best bet for our current
constraints.

The main difficulty with the second method is that we need to know the domain
of each computed. Unfortunately, even if we know the domains of the input and
output variables, the evolution process produces new computed with unknown
domains. Furthermore, this process is only moderately efficient if we can
constrain the domain of all computed in the evolution to values below some
small integer. I guess we'll make that another parameter of the evolution.
