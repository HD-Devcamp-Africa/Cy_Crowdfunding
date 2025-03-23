#![allow(clippy::arithmetic_side_effects)]

#[ink::contract]
mod crowdfunding {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::env::caller;

    #[ink(storage)]
    pub struct Crowdfunding {
        creator: AccountId,
        goal: Balance,
        deadline: BlockNumber,
        total_raised: Balance,
        contributions: Mapping<AccountId, Balance>,
        contributors: Vec<AccountId>,
    }

    #[ink(event)]
    pub struct CampaignCreated {
        #[ink(topic)]
        creator: AccountId,
        goal: Balance,
        deadline: BlockNumber,
    }

    #[ink(event)]
    pub struct ContributionMade {
        #[ink(topic)]
        contributor: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct RefundIssued {
        #[ink(topic)]
        contributor: AccountId,
        amount: Balance,
    }

    impl Crowdfunding {
        #[ink(constructor)]
        pub fn new(goal: Balance, deadline: BlockNumber) -> Self {
            let creator = caller::<ink::env::DefaultEnvironment>();
            let instance = Self {
                creator,
                goal,
                deadline,
                total_raised: 0,
                contributions: Mapping::default(),
                contributors: Vec::new(),
            };
            instance.env().emit_event(CampaignCreated {
                creator,
                goal,
                deadline,
            });
            instance
        }

        #[ink(message, payable)]
        pub fn contribute(&mut self) -> Result<(), String> {
            let contributor = caller::<ink::env::DefaultEnvironment>();
            let amount = self.env().transferred_value();

            if self.env().block_number() > self.deadline {
                return Err("Campaign deadline has passed".to_string());
            }

            let previous_amount = self.contributions.get(contributor).unwrap_or(0);
            self.total_raised = self.total_raised.checked_add(amount)
                .ok_or("Overflow occurred while adding to total_raised")?;

            self.contributions.insert(contributor, &(previous_amount + amount));
            if !self.contributors.contains(&contributor) {
                self.contributors.push(contributor);
            }

            self.env().emit_event(ContributionMade {
                contributor,
                amount,
            });
            Ok(())
        }

        pub fn is_successful(&self) -> bool {
            self.total_raised >= self.goal
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), String> {
            if caller::<ink::env::DefaultEnvironment>() != self.creator {
                return Err("Only the creator can withdraw funds".to_string());
            }
            if !self.is_successful() {
                return Err("Campaign did not meet its goal".to_string());
            }
            if self.env().block_number() <= self.deadline {
                return Err("Campaign deadline has not passed yet".to_string());
            }
            let amount = self.total_raised;
            self.env().transfer(self.creator, amount)
                .map_err(|e| format!("Transfer failed: {:?}", e))?;
            self.total_raised = 0;
            Ok(())
        }

        #[ink(message)]
        pub fn refund(&mut self) -> Result<(), String> {
            if self.env().block_number() <= self.deadline {
                return Err("Campaign deadline has not passed yet".to_string());
            }
            if self.is_successful() {
                return Err("Campaign was successful, no refunds needed".to_string());
            }
            let contributors = self.contributors.clone();
            for contributor in contributors {
                if let Some(amount) = self.contributions.get(contributor) {
                    if amount > 0 {
                        self.env().transfer(contributor, amount)
                            .map_err(|_| "Transfer failed")?;
                        self.contributions.remove(contributor);
                        self.env().emit_event(RefundIssued {
                            contributor,
                            amount,
                        });
                    }
                }
            }
            Ok(())
        }

        #[ink(message)]
        pub fn get_total_raised(&self) -> Balance {
            self.total_raised
        }

        #[ink(message)]
        pub fn get_deadline(&self) -> BlockNumber {
            self.deadline
        }

        #[ink(message)]
        pub fn get_goal(&self) -> Balance {
            self.goal
        }

        #[ink(message)]
        pub fn get_contributors(&self) -> Vec<AccountId> {
            self.contributors.clone()
        }
    }
}
