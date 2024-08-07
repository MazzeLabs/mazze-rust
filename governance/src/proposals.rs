
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub options: Vec<String>,
    pub status: ProposalStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Open,
    Closed,
}

impl Proposal {
    pub fn new(id: u64, title: &str, description: &str, options: Vec<String>) -> Self {
        Proposal {
            id,
            title: title.to_string(),
            description: description.to_string(),
            options,
            status: ProposalStatus::Open,
        }
    }

    pub fn close(&mut self) {
        self.status = ProposalStatus::Closed;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub proposal_id: u64,
    pub option: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalResult {
    pub proposal_id: u64,
    pub results: Vec<(String, u64)>, // (Option, Vote Count)
}

pub struct ProposalStore {
    proposals: Vec<Proposal>,
    votes: Vec<Vote>,
}

impl ProposalStore {
    pub fn new() -> Self {
        ProposalStore {
            proposals: Vec::new(),
            votes: Vec::new(),
        }
    }

    pub fn add_proposal(&mut self, proposal: Proposal) {
        self.proposals.push(proposal);
    }

    pub fn get_proposals(&self) -> &Vec<Proposal> {
        &self.proposals
    }

    pub fn cast_vote(&mut self, vote: Vote) {
        self.votes.push(vote);
    }

    pub fn get_results(&self, proposal_id: u64) -> Option<ProposalResult> {
        let proposal = self.proposals.iter().find(|p| p.id == proposal_id)?;
        if proposal.status == ProposalStatus::Open {
            return None;
        }

        let mut results = proposal
            .options
            .iter()
            .map(|option| (option.clone(), 0))
            .collect::<Vec<_>>();

        for vote in &self.votes {
            if vote.proposal_id == proposal_id {
                if let Some(option) = results.iter_mut().find(|(opt, _)| *opt == vote.option) {
                    option.1 += 1;
                }
            }
        }

        Some(ProposalResult {
            proposal_id,
            results,
        })
    }
}
