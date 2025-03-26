#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec, Map, Val, Error};

#[contract]
pub struct Crowdfunding;

#[contractimpl]
impl Crowdfunding {
    pub fn create_campaign(
        env: Env,
        creator: Address,
        goal: i128,
        deadline: u64,
    ) -> Result<u32, Error> {
        // Initialize storage if needed
        if !env.storage().has(&Symbol::new(&env, "campaign_count")) {
            env.storage().set(&Symbol::new(&env, "campaign_count"), &0u32);
        }

        // Get and increment campaign count
        let mut count: u32 = env.storage().get(&Symbol::new(&env, "campaign_count"))?.unwrap();
        let campaign_id = count;
        count += 1;
        env.storage().set(&Symbol::new(&env, "campaign_count"), &count);

        // Create and store campaign
        let campaign = Campaign {
            creator: creator.clone(),
            goal,
            deadline,
            total_raised: 0,
            contributions: Map::new(&env),
            contributors: Vec::new(&env),
        };
        env.storage().set(&campaign_key(&env, campaign_id), &campaign);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "campaign_created"), campaign_id),
            (creator, goal, deadline),
        );

        Ok(campaign_id)
    }

    pub fn contribute(
        env: Env,
        contributor: Address,
        campaign_id: u32,
        amount: i128,
    ) -> Result<(), Error> {
        let mut campaign: Campaign = get_campaign(&env, campaign_id)?;

        // Check campaign deadline
        if env.ledger().timestamp() > campaign.deadline {
            return Err(Error::from_contract_error(1)); // Custom error code
        }

        // Update contribution
        let current = campaign.contributions.get(contributor.clone()).unwrap_or(0);
        campaign.contributions.set(contributor.clone(), current + amount);
        campaign.total_raised += amount;

        // Add to contributors list if new
        if !campaign.contributors.contains(contributor.clone()) {
            campaign.contributors.push_back(contributor.clone());
        }

        // Save updated campaign
        env.storage().set(&campaign_key(&env, campaign_id), &campaign);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "contribution_made"), campaign_id),
            (contributor, amount),
        );

        Ok(())
    }

    pub fn withdraw(env: Env, caller: Address, campaign_id: u32) -> Result<(), Error> {
        let mut campaign: Campaign = get_campaign(&env, campaign_id)?;

        // Validate caller is creator
        if caller != campaign.creator {
            return Err(Error::from_contract_error(2)); // Unauthorized
        }

        // Validate campaign succeeded
        if campaign.total_raised < campaign.goal {
            return Err(Error::from_contract_error(3)); // Goal not reached
        }

        // Validate deadline passed
        if env.ledger().timestamp() <= campaign.deadline {
            return Err(Error::from_contract_error(4)); // Deadline not passed
        }

        // Transfer funds (simplified for example)
        // In real implementation, you'd use the token contract
        env.transfer(campaign.creator, campaign.total_raised)?;

        // Update campaign state
        campaign.total_raised = 0;
        env.storage().set(&campaign_key(&env, campaign_id), &campaign);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "funds_withdrawn"), campaign_id),
            (caller, campaign.total_raised),
        );

        Ok(())
    }

    pub fn refund(env: Env, caller: Address, campaign_id: u32) -> Result<(), Error> {
        let mut campaign: Campaign = get_campaign(&env, campaign_id)?;

        // Validate deadline passed
        if env.ledger().timestamp() <= campaign.deadline {
            return Err(Error::from_contract_error(4)); // Deadline not passed
        }

        // Validate campaign failed
        if campaign.total_raised >= campaign.goal {
            return Err(Error::from_contract_error(5)); // Campaign succeeded
        }

        // Refund all contributors
        for i in 0..campaign.contributors.len() {
            let contributor = campaign.contributors.get(i).unwrap();
            let amount = campaign.contributions.get(contributor.clone()).unwrap_or(0);
            
            if amount > 0 {
                env.transfer(contributor.clone(), amount)?;
                campaign.contributions.set(contributor.clone(), 0);
            }
        }

        // Update campaign state
        campaign.total_raised = 0;
        env.storage().set(&campaign_key(&env, campaign_id), &campaign);

        Ok(())
    }

    pub fn get_campaign_info(env: Env, campaign_id: u32) -> Result<CampaignInfo, Error> {
        let campaign: Campaign = get_campaign(&env, campaign_id)?;
        Ok(CampaignInfo {
            creator: campaign.creator,
            goal: campaign.goal,
            deadline: campaign.deadline,
            total_raised: campaign.total_raised,
            contributor_count: campaign.contributors.len() as u32,
        })
    }
}

// Helper functions
fn campaign_key(env: &Env, id: u32) -> Symbol {
    Symbol::new(env, &format!("campaign_{}", id))
}

fn get_campaign(env: &Env, id: u32) -> Result<Campaign, Error> {
    env.storage()
        .get(&campaign_key(env, id))?
        .ok_or(Error::from_contract_error(6)) // Campaign not found
}

// Data structures
#[derive(Clone, Debug, Eq, PartialEq, soroban_sdk::TryFromVal, soroban_sdk::IntoVal)]
struct Campaign {
    creator: Address,
    goal: i128,
    deadline: u64,
    total_raised: i128,
    contributions: Map<Address, i128>,
    contributors: Vec<Address>,
}

#[derive(Clone, Debug, Eq, PartialEq, soroban_sdk::TryFromVal, soroban_sdk::IntoVal)]
pub struct CampaignInfo {
    pub creator: Address,
    pub goal: i128,
    pub deadline: u64,
    pub total_raised: i128,
    pub contributor_count: u32,
}
