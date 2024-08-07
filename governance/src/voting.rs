use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub voter_id: String,
    pub proposal_id: u64,
    pub option: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Voter {
    pub id: String,
    pub name: String,
}

pub struct VotingSystem {
    votes: Vec<Vote>,
    voters: HashMap<String, Voter>,
}

impl VotingSystem {
    pub fn new() -> Self {
        VotingSystem {
            votes: Vec::new(),
            voters: HashMap::new(),
        }
    }

    pub fn register_voter(&mut self, id: &str, name: &str) {
        let voter = Voter {
            id: id.to_string(),
            name: name.to_string(),
        };
        self.voters.insert(id.to_string(), voter);
    }

    pub fn cast_vote(&mut self, voter_id: &str, proposal_id: u64, option: &str) -> Result<(), &'static str> {
        if !self.voters.contains_key(voter_id) {
            return Err("Voter not registered");
        }
        let vote = Vote {
            voter_id: voter_id.to_string(),
            proposal_id,
            option: option.to_string(),
        };
        self.votes.push(vote);
        Ok(())
    }

    pub fn tally_votes(&self, proposal_id: u64) -> HashMap<String, u64> {
        let mut results = HashMap::new();
        for vote in &self.votes {
            if vote.proposal_id == proposal_id {
                *results.entry(vote.option.clone()).or_insert(0) += 1;
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voting_system() {
        let mut voting_system = VotingSystem::new();
        voting_system.register_voter("voter1", "Alice");
        voting_system.register_voter("voter2", "Bob");

        voting_system.cast_vote("voter1", 1, "Option A").unwrap();
        voting_system.cast_vote("voter2", 1, "Option B").unwrap();
        voting_system.cast_vote("voter1", 2, "Option C").unwrap();

        let results = voting_system.tally_votes(1);
        assert_eq!(results.get("Option A"), Some(&1));
        assert_eq!(results.get("Option B"), Some(&1));
        assert_eq!(results.get("Option C"), None);
    }

    #[test]
    fn test_unregistered_voter() {
        let mut voting_system = VotingSystem::new();
        let result = voting_system.cast_vote("voter1", 1, "Option A");
        assert!(result.is_err());
    }
}

