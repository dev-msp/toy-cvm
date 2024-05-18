use std::{collections::HashSet, fmt::Debug, hash::Hash};

use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};

trait Element: Clone + Hash + PartialEq + Eq + Debug {}
impl<T: Clone + Hash + PartialEq + Eq + Debug> Element for T {}

/// The main data structure for the [CVM algorithm](https://arxiv.org/abs/2301.10191).
#[derive(Debug)]
struct Cvm<T> {
    capacity: usize,
    memory: HashSet<T>,
    rounds: u32,
}

impl<T: Hash + PartialEq + Eq + Debug> Cvm<T> {
    fn new(capacity: usize) -> Self {
        Cvm {
            capacity,
            memory: HashSet::new(),
            rounds: 0,
        }
    }

    fn should_keep(coin_flips: u32) -> bool {
        (0..coin_flips).all(|_| rand::random())
    }

    fn estimate(&self) -> usize {
        let rounds = if self.rounds > 32 { 32 } else { self.rounds };
        self.memory.len() * 2_usize.pow(rounds)
    }

    fn extend<I>(&mut self, iter: I)
    where
        I: Iterator<Item = T>,
    {
        for i in iter {
            self.add(i);
        }
    }

    fn add(&mut self, value: T) {
        match self.rounds {
            0 => {
                self.memory.insert(value);
            }
            n if Self::should_keep(n) => {
                self.memory.insert(value);
            }
            _ if self.memory.contains(&value) => {
                self.memory.remove(&value);
            }
            _ => {}
        }

        if self.memory.len() >= self.capacity {
            self.sweep();
        }
    }

    fn sweep(&mut self) {
        self.memory.retain(|_| Self::should_keep(1));
        self.rounds += 1;
    }
}

/// Not taken from the paper, just me playing around.
struct CombinedCvm<T> {
    cvms: Vec<Cvm<T>>,
}

/// # Panics
///
/// Panics if `len` is 0.
impl<T: Element> CombinedCvm<T> {
    fn new(capacity: usize, len: usize) -> Self {
        if len == 0 {
            panic!("Length must be greater than 0");
        }
        CombinedCvm {
            cvms: (0..len).map(|_| Cvm::new(capacity)).collect(),
        }
    }

    fn extend<I>(&mut self, iter: I)
    where
        I: Iterator<Item = T>,
    {
        for i in iter {
            self.add(&i);
        }
    }

    fn add(&mut self, value: &T) {
        for c in self.cvms.iter_mut() {
            c.add(value.clone());
        }
    }

    fn estimate(&self) -> usize {
        let ests = self.cvms.iter().map(Cvm::estimate).collect::<Vec<_>>();
        // remove min and max
        let mut ests = ests;
        ests.sort();
        ests.pop();
        ests.remove(0);
        ests.iter().sum::<usize>() / ests.len()
    }
}

struct Test<I>
where
    I: Iterator,
    I::Item: Element,
{
    memory_capacity: usize,
    data: I,
    sample_size: usize,
    instances: Option<usize>,
}

fn run_test<I>(test: Test<I>) -> usize
where
    I: Iterator,
    I::Item: Element,
{
    let Some(instances) = test.instances else {
        let mut c = Cvm::new(test.memory_capacity);
        c.extend(test.data.take(test.sample_size));
        return c.estimate();
    };

    let mut c = CombinedCvm::new(test.memory_capacity, instances);
    c.extend(test.data.take(test.sample_size));
    c.estimate()
}

fn main() {
    println!(
        "Result: {}",
        run_test(Test {
            memory_capacity: 1000,
            data: StdRng::from_entropy().sample_iter(Uniform::new(0, 10000)),
            sample_size: 30000,
            instances: None
        })
    );
}
