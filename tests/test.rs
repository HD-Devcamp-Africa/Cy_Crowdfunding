#![cfg(test)]
use soroban_sdk::{Env, Address, Error, Symbol, Val};

use crate::{Crowdfunding, CrowdfundingClient, CampaignInfo};

#[test]
fn test_create_campaign() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Crowdfunding);
    let client = CrowdfundingClient::new(&env, &contract_id);

    let creator = Address::random(&env);
    let goal = 1000;
    let deadline = 100;

    // Create first campaign
    let campaign_id = client.create_campaign(&creator, &goal, &deadline);
    assert_eq!(campaign_id, 0);

    // Verify campaign info
    let info = client.get_campaign_info(&campaign_id);
    assert_eq!(info.creator, creator);
    assert_eq!(info.goal, goal);
    assert_eq!(info.deadline, deadline);
    assert_eq!(info.total_raised, 0);
    assert_eq!(info.contributor_count, 0);

    // Check event
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].0,
        (Symbol::new(&env, "campaign_created"), 0u32).into_val(&env)
    );
}

#[test]
fn test_contribute() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Crowdfunding);
    let client = CrowdfundingClient::new(&env, &contract_id);

    let creator = Address::random(&env);
    let contributor = Address::random(&env);
    let goal = 1000;
    let deadline = 100;

    // Create campaign
    let campaign_id = client.create_campaign(&creator, &goal, &deadline);

    // Contribute
    let amount = 500;
    client.contribute(&contributor, &campaign_id, &amount);

    // Verify campaign info
    let info = client.get_campaign_info(&campaign_id);
    assert_eq!(info.total_raised, amount);
    assert_eq!(info.contributor_count, 1);

    // Check event
    let events = env.events().all();
    assert_eq!(events.len(), 2); // creation + contribution
}

#[test]
fn test_withdraw_success() {
    let env = Env::default();
    env.ledger().set_timestamp(50); // Set initial time
    let contract_id = env.register_contract(None, Crowdfunding);
    let client = CrowdfundingClient::new(&env, &contract_id);

    let creator = Address::random(&env);
    let goal = 1000;
    let deadline = 100;

    // Create campaign
    let campaign_id = client.create_campaign(&creator, &goal, &deadline);

    // Fully fund the campaign
    client.contribute(&creator, &campaign_id, &goal);

    // Advance time past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Withdraw funds
    client.withdraw(&creator, &campaign_id);

    // Verify campaign state
    let info = client.get_campaign_info(&campaign_id);
    assert_eq!(info.total_raised, 0);
}

#[test]
fn test_refund() {
    let env = Env::default();
    env.ledger().set_timestamp(50);
    let contract_id = env.register_contract(None, Crowdfunding);
    let client = CrowdfundingClient::new(&env, &contract_id);

    let creator = Address::random(&env);
    let contributor = Address::random(&env);
    let goal = 1000;
    let deadline = 100;

    // Create campaign
    let campaign_id = client.create_campaign(&creator, &goal, &deadline);

    // Partial contribution
    client.contribute(&contributor, &campaign_id, &500);

    // Advance time past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Request refund
    client.refund(&contributor, &campaign_id);

    // Verify campaign state
    let info = client.get_campaign_info(&campaign_id);
    assert_eq!(info.total_raised, 0);
}

#[test]
#[should_panic(expected = "Error(ContractError(1))")] // Deadline passed
fn test_contribute_after_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(50);
    let contract_id = env.register_contract(None, Crowdfunding);
    let client = CrowdfundingClient::new(&env, &contract_id);

    let creator = Address::random(&env);
    let goal = 1000;
    let deadline = 100;

    // Create campaign with short deadline
    let campaign_id = client.create_campaign(&creator, &goal, &deadline);

    // Advance time past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Try to contribute (should fail)
    client.contribute(&creator, &campaign_id, &500);
}
