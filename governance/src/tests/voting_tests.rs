#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposals::{Proposal, ProposalStatus};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_create_vote() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;

        let vote = Vote {
            proposal_id: 1,
            voter_id: "voter1".to_string(),
            choice: VoteChoice::Yes,
        };

        let result = voting_system.cast_vote(vote).await;

        assert!(result.is_ok());
        assert_eq!(voting_system.proposals.lock().await.get(&1).unwrap().votes.len(), 1);
    }

    #[tokio::test]
    async fn test_vote_tallying() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;

        let votes = vec![
            Vote {
                proposal_id: 1,
                voter_id: "voter1".to_string(),
                choice: VoteChoice::Yes,
            },
            Vote {
                proposal_id: 1,
                voter_id: "voter2".to_string(),
                choice: VoteChoice::No,
            },
            Vote {
                proposal_id: 1,
                voter_id: "voter3".to_string(),
                choice: VoteChoice::Yes,
            },
        ];

        for vote in votes {
            voting_system.cast_vote(vote).await.unwrap();
        }

        let tally = voting_system.tally_votes(1).await.unwrap();

        assert_eq!(tally.yes, 2);
        assert_eq!(tally.no, 1);
    }

    #[tokio::test]
    async fn test_vote_on_nonexistent_proposal() {
        let mut voting_system = VotingSystem::new();

        let vote = Vote {
            proposal_id: 999,
            voter_id: "voter1".to_string(),
            choice: VoteChoice::Yes,
        };

        let result = voting_system.cast_vote(vote).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Proposal does not exist");
    }

    #[tokio::test]
    async fn test_duplicate_vote() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;

        let vote = Vote {
            proposal_id: 1,
            voter_id: "voter1".to_string(),
            choice: VoteChoice::Yes,
        };

        voting_system.cast_vote(vote.clone()).await.unwrap();
        let result = voting_system.cast_vote(vote).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Voter has already voted");
    }

    #[tokio::test]
    async fn test_close_voting() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;

        let vote = Vote {
            proposal_id: 1,
            voter_id: "voter1".to_string(),
            choice: VoteChoice::Yes,
        };

        voting_system.cast_vote(vote).await.unwrap();
        voting_system.close_voting(1).await.unwrap();

        let proposal = voting_system.proposals.lock().await.get(&1).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Closed);
    }

    #[tokio::test]
    async fn test_voting_on_closed_proposal() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;

        voting_system.close_voting(1).await.unwrap();

        let vote = Vote {
            proposal_id: 1,
            voter_id: "voter1".to_string(),
            choice: VoteChoice::Yes,
        };

        let result = voting_system.cast_vote(vote).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Voting is closed for this proposal");
    }

    #[tokio::test]
    async fn test_remove_proposal() {
        let mut voting_system = VotingSystem::new();

        let proposal = Proposal::new(1, "Test Proposal".to_string(), "Description".to_string());
        voting_system.add_proposal(proposal).await;
        
        voting_system.remove_proposal(1).await.unwrap();

        assert!(voting_system.proposals.lock().await.get(&1).is_none());
    }
}
