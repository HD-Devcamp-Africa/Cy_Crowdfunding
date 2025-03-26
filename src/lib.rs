#![allow(clippy::arithmetic_side_effects)]

#[ink::contract]
mod crowdfunding {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::env::caller;

    #[derive(scale::Encode, scale::Decode, Clone, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Campaign {
        creator: AccountId,
        goal: Balance,
        deadline: BlockNumber,
        total_raised: Balance,
        contributions: Mapping<AccountId, Balance>,
        contributors: Vec<AccountId>,
    }

    #[ink(storage)]
    pub struct Crowdfunding {
        campaigns: Mapping<u32, Campaign>,
        campaign_count: u32,
    }

    #[ink(event)]
    pub struct CampaignCreated {
        #[ink(topic)]
        campaign_id: u32,
        creator: AccountId,
        goal: Balance,
        deadline: BlockNumber,
    }

    #[ink(event)]
    pub struct ContributionMade {
        #[ink(topic)]
        campaign_id: u32,
        contributor: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct RefundIssued {
        #[ink(topic)]
        campaign_id: u32,
        contributor: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct FundsWithdrawn {
        #[ink(topic)]
        campaign_id: u32,
        creator: AccountId,
        amount: Balance,
    }

    impl Crowdfunding {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                campaigns: Mapping::default(),
                campaign_count: 0,
            }
        }

        #[ink(message)]
        pub fn create_campaign(&mut self, goal: Balance, deadline: BlockNumber) -> u32 {
            let creator = caller::<ink::env::DefaultEnvironment>();
            let campaign_id = self.campaign_count;
            let campaign = Campaign {
                creator,
                goal,
                deadline,
                total_raised: 0,
                contributions: Mapping::default(),
                contributors: Vec::new(),
            };

            self.campaigns.insert(campaign_id, &campaign);
            self.campaign_count += 1;

            self.env().emit_event(CampaignCreated {
                campaign_id,
                creator,
                goal,
                deadline,
            });

            campaign_id
        }

        #[ink(message, payable)]
        pub fn contribute(&mut self, campaign_id: u32) -> Result<(), String> {
            let mut campaign = self.campaigns.get(campaign_id).ok_or("Campaign not found")?;
            let contributor = caller::<ink::env::DefaultEnvironment>();
            let amount = self.env().transferred_value();

            if self.env().block_number() > campaign.deadline {
                return Err("Campaign deadline has passed".to_string());
            }

            let previous_amount = campaign.contributions.get(contributor).unwrap_or(0);
            campaign.total_raised = campaign.total_raised.checked_add(amount)
                .ok_or("Overflow occurred while adding to total_raised")?;

            campaign.contributions.insert(contributor, &(previous_amount + amount));
            if !campaign.contributors.contains(&contributor) {
                campaign.contributors.push(contributor);
            }

            self.env().emit_event(ContributionMade {
                campaign_id,
                contributor,
                amount,
            });

            self.campaigns.insert(campaign_id, &campaign);
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, campaign_id: u32) -> Result<(), String> {
            let mut campaign = self.campaigns.get(campaign_id).ok_or("Campaign not found")?;
            let creator = caller::<ink::env::DefaultEnvironment>();

            if creator != campaign.creator {
                return Err("Only the campaign creator can withdraw funds".to_string());
            }
            if campaign.total_raised < campaign.goal {
                return Err("Campaign did not reach its goal".to_string());
            }
            if self.env().block_number() <= campaign.deadline {
                return Err("Campaign deadline has not passed yet".to_string());
            }

            let amount = campaign.total_raised;
            self.env().transfer(campaign.creator, amount)
                .map_err(|_| "Transfer failed")?;
            
            campaign.total_raised = 0;
            self.campaigns.insert(campaign_id, &campaign);

            self.env().emit_event(FundsWithdrawn {
                campaign_id,
                creator,
                amount,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn refund(&mut self, campaign_id: u32) -> Result<(), String> {
            let mut campaign = self.campaigns.get(campaign_id).ok_or("Campaign not found")?;

            if self.env().block_number() <= campaign.deadline {
                return Err("Campaign deadline has not passed yet".to_string());
            }
            if campaign.total_raised >= campaign.goal {
                return Err("Campaign was successful, no refunds needed".to_string());
            }

            let contributors = campaign.contributors.clone();
            for contributor in contributors {
                if let Some(amount) = campaign.contributions.get(contributor) {
                    if amount > 0 {
                        self.env().transfer(contributor, amount)
                            .map_err(|_| "Transfer failed")?;
                        campaign.contributions.remove(contributor);
                        self.env().emit_event(RefundIssued {
                            campaign_id,
                            contributor,
                            amount,
                        });
                    }
                }
            }

            self.campaigns.insert(campaign_id, &campaign);
            Ok(())
        }

        #[ink(message)]
        pub fn is_goal_reached(&self, campaign_id: u32) -> bool {
            if let Some(campaign) = self.campaigns.get(campaign_id) {
                return campaign.total_raised >= campaign.goal;
            }
            false
        }

        #[ink(message)]
        pub fn get_all_campaigns(&self) -> Vec<(u32, AccountId, Balance, BlockNumber, Balance)> {
            let mut all_campaigns = Vec::new();
            for campaign_id in 0..self.campaign_count {
                if let Some(campaign) = self.campaigns.get(campaign_id) {
                    all_campaigns.push((
                        campaign_id,
                        campaign.creator,
                        campaign.goal,
                        campaign.deadline,
                        campaign.total_raised,
                    ));
                }
            }
            all_campaigns
        }
    }
}
