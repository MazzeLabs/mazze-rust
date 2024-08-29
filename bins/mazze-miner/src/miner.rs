use mazze_types::{H256, U256};
use mazzecore::pow::{ProofOfWorkProblem, ProofOfWorkSolution, PowComputer};
use std::time::{Duration, Instant};

pub struct Miner {
    pow_computer: PowComputer,
}

impl Miner {
    pub fn new() -> Self {
        Miner {
            pow_computer: PowComputer::new(),
        }
    }

    pub fn mine(&self, problem: &ProofOfWorkProblem, timeout: Duration) -> Option<ProofOfWorkSolution> {
        self.pow_computer.initialize(&problem.block_hash);
        self.pow_computer.mine(problem, timeout)
    }
}