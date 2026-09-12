use anyhow::Result;

pub enum BisectStrategy {
    Linear,
    Binary,
    Custom(Vec<String>),
}

impl BisectStrategy {
    pub fn calculate_midpoint(&self, start: usize, end: usize) -> usize {
        match self {
            BisectStrategy::Linear => start + 1,
            BisectStrategy::Binary => (start + end) / 2,
            BisectStrategy::Custom(_) => (start + end) / 2,
        }
    }

    pub fn get_commits(&self) -> Vec<String> {
        match self {
            BisectStrategy::Custom(commits) => commits.clone(),
            _ => Vec::new(),
        }
    }
}

pub struct BisectPlanner {
    start_commit: String,
    end_commit: String,
    strategy: BisectStrategy,
}

impl BisectPlanner {
    pub fn new(start_commit: &str, end_commit: &str) -> Self {
        Self {
            start_commit: start_commit.to_string(),
            end_commit: end_commit.to_string(),
            strategy: BisectStrategy::Binary,
        }
    }

    pub fn with_strategy(mut self, strategy: BisectStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn plan(&self) -> BisectPlan {
        let estimated_steps = match &self.strategy {
            BisectStrategy::Binary => {
                // Binary search: log2(n) steps
                let n = 100; // Estimated number of commits
                (n as f64).log2().ceil() as usize
            }
            BisectStrategy::Linear => 50, // Average case
            BisectStrategy::Custom(commits) => commits.len(),
        };

        BisectPlan {
            start_commit: self.start_commit.clone(),
            end_commit: self.end_commit.clone(),
            strategy: self.strategy.clone(),
            estimated_steps,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BisectPlan {
    pub start_commit: String,
    pub end_commit: String,
    pub strategy: BisectStrategy,
    pub estimated_steps: usize,
}

impl std::fmt::Display for BisectPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bisect Plan:\n  Start: {}\n  End: {}\n  Strategy: {:?}\n  Estimated steps: {}",
            self.start_commit, self.end_commit, self.strategy, self.estimated_steps
        )
    }
}
