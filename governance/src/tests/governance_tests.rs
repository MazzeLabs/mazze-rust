#[cfg(test)]
mod tests {
    use super::*;
    use crate::proposals::{Proposal, ProposalStatus};
    use crate::voting::{Vote, Voter};
    use std::collections::HashMap;

    #[test]
    fn test_create_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal::new("Test Proposal".to_string(), "Description of the proposal".to_string());
        
        governance.create_proposal(proposal.clone());
        assert_eq!(governance.proposals.len(), 1);
        assert_eq!(governance.proposals[0].title, "Test Proposal");
        assert_eq!(governance.proposals[0].status, ProposalStatus::Pending);
    }

    #[test]
    fn test_vote_on_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal::new("Test Proposal".to_string(), "Description of the proposal".to_string());
        
        governance.create_proposal(proposal.clone());
        let voter = Voter::new("voter1".to_string(), 100);
        
        governance.vote_on_proposal(0, Vote::Approve, voter.clone());
        
        assert_eq!(governance.proposals[0].votes.len(), 1);
        assert_eq!(governance.proposals[0].votes[0].voter.id, "voter1");
        assert_eq!(governance.proposals[0].votes[0].vote, Vote::Approve);
    }

    #[test]
    fn test_calculate_votes() {
        let mut governance = Governance::new();
        let proposal = Proposal::new("Test Proposal".to_string(), "Description of the proposal".to_string());
        
        governance.create_proposal(proposal.clone());
        let voter1 = Voter::new("voter1".to_string(), 100);
        let voter2 = Voter::new("voter2".to_string(), 200);
        
        governance.vote_on_proposal(0, Vote::Approve, voter1.clone());
        governance.vote_on_proposal(0, Vote::Reject, voter2.clone());
        
        let result = governance.calculate_votes(0).unwrap();
        
        assert_eq!(result.approve, 100);
        assert_eq!(result.reject, 200);
        assert_eq!(result.total_votes, 300);
    }

    #[test]
    fn test_finalize_proposal() {
        let mut governance = Governance::new();
        let proposal = Proposal::new("Test Proposal".to_string(), "Description of the proposal".to_string());
        
        governance.create_proposal(proposal.clone());
        let voter1 = Voter::new("voter1".to_string(), 100);
        let voter2 = Voter::new("voter2".to_string(), 200);
        
        governance.vote_on_proposal(0, Vote::Approve, voter1.clone());
        governance.vote_on_proposal(0, Vote::Approve, voter2.clone());
        
        governance.finalize_proposal(0);
        
        assert_eq!(governance.proposals[0].status, ProposalStatus::Approved);
    }

    #[test]
    fn test_analyze_proposals() {
        let mut governance = Governance::new();
        
        let proposal1 = Proposal::new("Proposal 1".to_string(), "Description 1".to_string());
        let proposal2 = Proposal::new("Proposal 2".to_string(), "Description 2".to_string());
        
        governance.create_proposal(proposal1.clone());
        governance.create_proposal(proposal2.clone());
        
        let analysis = governance.analyze_proposals();
        
        assert_eq!(analysis.len(), 2);
        assert_eq!(analysis[0].title, "Proposal 1");
        assert_eq!(analysis[1].title, "Proposal 2");
    }

    #[test]
    fn test_governance_history() {
        let mut governance = Governance::new();
        
        let proposal = Proposal::new("Historical Proposal".to_string(), "Description".to_string());
        
        governance.create_proposal(proposal.clone());
        let voter = Voter::new("voter".to_string(), 50);
        
        governance.vote_on_proposal(0, Vote::Approve, voter.clone());
        governance.finalize_proposal(0);
        
        let history = governance.governance_history();
        
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].proposal.title, "Historical Proposal");
        assert_eq!(history[0].proposal.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_governance_participants() {
        let mut governance = Governance::new();
        
        let voter1 = Voter::new("voter1".to_string(), 50);
        let voter2 = Voter::new("voter2".to_string(), 100);
        
        governance.register_voter(voter1.clone());
        governance.register_voter(voter2.clone());
        
        let participants = governance.governance_participants();
        
        assert_eq!(participants.len(), 2);
        assert!(participants.contains(&voter1));
        assert!(participants.contains(&voter2));
    }
}
