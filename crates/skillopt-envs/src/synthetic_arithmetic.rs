use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use skillopt_core::{Environment, Example};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SyntheticArithmeticParams {
    pub train_size: usize,
    pub val_size: usize,
    pub test_size: usize,
    pub seed: u64,
    /// Fraction of problems (0.0-1.0) that include an irrelevant distractor
    /// number, e.g. "Alice is 12. Bob has 3 apples and buys 4 more." — the
    /// distractor (12) shouldn't affect the answer. This is where a naive
    /// initial skill tends to fail and gives the optimizer something real
    /// to fix.
    pub distractor_rate: f64,
}

impl Default for SyntheticArithmeticParams {
    fn default() -> Self {
        Self { train_size: 24, val_size: 8, test_size: 16, seed: 0, distractor_rate: 0.5 }
    }
}

/// A small, fully offline, deterministically-generated word-problem
/// benchmark. Standing in for SkillOpt's real benchmark adapters (e.g.
/// SearchQA): programmatic scoring means the validation gate never itself
/// depends on an LLM call, and generation needs no network or dataset file.
pub struct SyntheticArithmeticEnv {
    train: Vec<Example>,
    val: Vec<Example>,
    test: Vec<Example>,
}

impl SyntheticArithmeticEnv {
    pub fn new(params: SyntheticArithmeticParams) -> Self {
        let mut rng = StdRng::seed_from_u64(params.seed);
        let mut next_id = 0usize;
        let mut gen = |rng: &mut StdRng, n: usize, prefix: &str| -> Vec<Example> {
            (0..n)
                .map(|_| {
                    let ex = gen_example(rng, params.distractor_rate, next_id, prefix);
                    next_id += 1;
                    ex
                })
                .collect()
        };
        let train = gen(&mut rng, params.train_size, "train");
        let val = gen(&mut rng, params.val_size, "val");
        let test = gen(&mut rng, params.test_size, "test");
        Self { train, val, test }
    }
}

fn gen_example(rng: &mut StdRng, distractor_rate: f64, id: usize, prefix: &str) -> Example {
    let names = ["Alice", "Bob", "Carla", "Dev", "Ewa", "Femi"];
    let items = ["apples", "marbles", "stickers", "coins", "books"];

    let name = names[rng.gen_range(0..names.len())];
    let item = items[rng.gen_range(0..items.len())];
    let start: i64 = rng.gen_range(1..30);
    let delta: i64 = rng.gen_range(-10..15);
    let op_is_gain = delta >= 0;

    let mut input = if op_is_gain {
        format!("{name} has {start} {item} and gets {delta} more {item}.")
    } else {
        format!("{name} has {start} {item} and gives away {} {item}.", -delta)
    };

    if rng.gen_bool(distractor_rate) {
        let distractor_name = names[rng.gen_range(0..names.len())];
        let distractor_age: i64 = rng.gen_range(5..80);
        input.push_str(&format!(" {distractor_name} is {distractor_age} years old."));
    }

    input.push_str(&format!(" How many {item} does {name} have now?"));

    let expected = (start + delta).to_string();

    Example { id: format!("{prefix}-{id}"), input, expected }
}

impl Environment for SyntheticArithmeticEnv {
    fn name(&self) -> &str {
        "synthetic_arithmetic"
    }
    fn train_examples(&self) -> &[Example] {
        &self.train
    }
    fn val_examples(&self) -> &[Example] {
        &self.val
    }
    fn test_examples(&self) -> &[Example] {
        &self.test
    }

    /// Lenient-but-strict scoring: extracts the *last* integer literal in
    /// the output (tolerating surrounding prose like "The answer is 7.")
    /// and compares it exactly to the expected integer.
    fn score(&self, example: &Example, output: &str) -> f64 {
        let expected: i64 = match example.expected.parse() {
            Ok(v) => v,
            Err(_) => return 0.0,
        };
        match last_integer(output) {
            Some(actual) if actual == expected => 1.0,
            _ => 0.0,
        }
    }
}

fn last_integer(text: &str) -> Option<i64> {
    let mut found: Option<i64> = None;
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        let negative = bytes[i] == '-' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit();
        let start = if negative { i + 1 } else { i };
        if bytes.get(start).is_some_and(|c| c.is_ascii_digit()) {
            let mut j = start;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            let digits: String = bytes[start..j].iter().collect();
            if let Ok(mut v) = digits.parse::<i64>() {
                if negative {
                    v = -v;
                }
                found = Some(v);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_deterministic_given_seed() {
        let a = SyntheticArithmeticEnv::new(SyntheticArithmeticParams { seed: 5, ..Default::default() });
        let b = SyntheticArithmeticEnv::new(SyntheticArithmeticParams { seed: 5, ..Default::default() });
        assert_eq!(
            a.train.iter().map(|e| &e.input).collect::<Vec<_>>(),
            b.train.iter().map(|e| &e.input).collect::<Vec<_>>()
        );
    }

    #[test]
    fn splits_have_requested_sizes_and_no_overlap() {
        let env = SyntheticArithmeticEnv::new(SyntheticArithmeticParams {
            train_size: 5,
            val_size: 2,
            test_size: 3,
            seed: 1,
            distractor_rate: 0.5,
        });
        assert_eq!(env.train_examples().len(), 5);
        assert_eq!(env.val_examples().len(), 2);
        assert_eq!(env.test_examples().len(), 3);
        let mut ids: Vec<&str> = env
            .train_examples()
            .iter()
            .chain(env.val_examples())
            .chain(env.test_examples())
            .map(|e| e.id.as_str())
            .collect();
        let before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), before, "example ids must be unique across splits");
    }

    #[test]
    fn scores_exact_answer_amid_prose() {
        let env = SyntheticArithmeticEnv::new(SyntheticArithmeticParams::default());
        let ex = Example { id: "x".into(), input: String::new(), expected: "7".into() };
        assert_eq!(env.score(&ex, "The answer is 7."), 1.0);
        assert_eq!(env.score(&ex, "7"), 1.0);
        assert_eq!(env.score(&ex, "I think it's 8, definitely 8."), 0.0);
        assert_eq!(env.score(&ex, "no numbers here"), 0.0);
    }

    #[test]
    fn scores_negative_answers() {
        let env = SyntheticArithmeticEnv::new(SyntheticArithmeticParams::default());
        let ex = Example { id: "x".into(), input: String::new(), expected: "-3".into() };
        assert_eq!(env.score(&ex, "The result is -3."), 1.0);
        assert_eq!(env.score(&ex, "The result is 3."), 0.0);
    }

    #[test]
    fn expected_answers_match_generated_arithmetic() {
        let env = SyntheticArithmeticEnv::new(SyntheticArithmeticParams { seed: 3, ..Default::default() });
        for e in env.train_examples() {
            // Every generated expected value must itself be parseable and
            // recoverable via the same scoring function (sanity check that
            // the scorer and generator agree).
            let expected: i64 = e.expected.parse().unwrap();
            assert_eq!(env.score(e, &e.expected), 1.0, "expected {expected} should self-score 1.0");
        }
    }
}
