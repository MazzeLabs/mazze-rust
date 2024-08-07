#[cfg(test)]
mod proposals_tests {
    use super::*;
    use crate::proposals::{Proposal, ProposalStatus, GovernanceError};
    use crate::voting::Vote;
    use std::collections::HashMap;

    #[test]
    fn test_create_proposal() {
        let title = "Increase block size".to_string();
        let description = "Proposal to increase the block size from 1MB to 2MB".to_string();
        let proposal = Proposal::new(title.clone(), description.clone());

        assert_eq!(proposal.title, title);
        assert_eq!(proposal.description, description);
        assert_eq!(proposal.status, ProposalStatus::Pending);
        assert!(proposal.votes.is_empty());
    }

    #[test]
    fn test_add_vote() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let voter_id = "voter_123";
        let vote = Vote::Yes;
        proposal.add_vote(voter_id, vote.clone()).unwrap();

        assert_eq!(proposal.votes.len(), 1);
        assert_eq!(proposal.votes.get(voter_id).unwrap(), &vote);
    }

    #[test]
    fn test_add_vote_duplicate() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let voter_id = "voter_123";
        let vote = Vote::Yes;
        proposal.add_vote(voter_id, vote.clone()).unwrap();

        let result = proposal.add_vote(voter_id, Vote::No);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), GovernanceError::DuplicateVote);
    }

    #[test]
    fn test_tally_votes() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let votes = vec![
            ("voter_1", Vote::Yes),
            ("voter_2", Vote::No),
            ("voter_3", Vote::Yes),
            ("voter_4", Vote::Yes),
            ("voter_5", Vote::No),
        ];

        for (voter_id, vote) in votes {
            proposal.add_vote(voter_id, vote).unwrap();
        }

        let tally = proposal.tally_votes();
        assert_eq!(tally.get(&Vote::Yes).cloned().unwrap_or(0), 3);
        assert_eq!(tally.get(&Vote::No).cloned().unwrap_or(0), 2);
    }

    #[test]
    fn test_finalize_proposal() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let votes = vec![
            ("voter_1", Vote::Yes),
            ("voter_2", Vote::No),
            ("voter_3", Vote::Yes),
            ("voter_4", Vote::Yes),
            ("voter_5", Vote::No),
        ];

        for (voter_id, vote) in votes {
            proposal.add_vote(voter_id, vote).unwrap();
        }

        proposal.finalize().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_finalize_proposal_tie() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let votes = vec![
            ("voter_1", Vote::Yes),
            ("voter_2", Vote::No),
            ("voter_3", Vote::Yes),
            ("voter_4", Vote::No),
        ];

        for (voter_id, vote) in votes {
            proposal.add_vote(voter_id, vote).unwrap();
        }

        proposal.finalize().unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_finalize_proposal_without_votes() {
        let mut proposal = Proposal::new(
            "Increase block size".to_string(),
            "Proposal to increase the block size from 1MB to 2MB".to_string(),
        );

        let result = proposal.finalize();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), GovernanceError::NoVotes);
    }
}
