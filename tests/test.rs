#[cfg(test)]
mod tests {
    use ink_e2e::build_message;

    use crowdfunding::Crowdfunding;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn test_create_campaign(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Deploy the contract
        let constructor = Crowdfunding::new(1000, 100); // Goal: 1000, Deadline: Block 100
        let contract = client
            .instantiate("crowdfunding", &ink_e2e::alice(), constructor)
            .submit()
            .await
            .expect("Failed to instantiate contract");
        let mut call = contract.call::<Crowdfunding>();

        // Check initial state
        let goal = call.get_goal().dry_run().await?;
        assert_eq!(goal, 1000);

        let deadline = call.get_deadline().dry_run().await?;
        assert_eq!(deadline, 100);

        let total_raised = call.get_total_raised().dry_run().await?;
        assert_eq!(total_raised, 0);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_contribute(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Deploy the contract
        let constructor = Crowdfunding::new(1000, 100); // Goal: 1000, Deadline: Block 100
        let contract = client
            .instantiate("crowdfunding", &ink_e2e::alice(), constructor)
            .submit()
            .await
            .expect("Failed to instantiate contract");
        let mut call = contract.call::<Crowdfunding>();

        // Alice contributes 500
        let contribute = build_message::<Crowdfunding>(contract.account_id.clone())
            .call(|crowdfunding| crowdfunding.contribute());
        client
            .call(&ink_e2e::alice(), contribute, 500, None)
            .await
            .expect("Failed to call contribute");

        // Check total raised
        let total_raised = call.get_total_raised().dry_run().await?;
        assert_eq!(total_raised, 500);

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_withdraw_success(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Deploy the contract
        let constructor = Crowdfunding::new(1000, 100); // Goal: 1000, Deadline: Block 100
        let contract = client
            .instantiate("crowdfunding", &ink_e2e::alice(), constructor)
            .submit()
            .await
            .expect("Failed to instantiate contract");
        let mut call = contract.call::<Crowdfunding>();

        // Alice contributes 1000 (meets the goal)
        let contribute = build_message::<Crowdfunding>(contract.account_id.clone())
            .call(|crowdfunding| crowdfunding.contribute());
        client
            .call(&ink_e2e::alice(), contribute, 1000, None)
            .await
            .expect("Failed to call contribute");

        // Fast-forward to after the deadline
        client
            .advance_block()
            .await
            .expect("Failed to advance block");

        // Alice withdraws funds
        let withdraw = build_message::<Crowdfunding>(contract.account_id.clone())
            .call(|crowdfunding| crowdfunding.withdraw());
        client
            .call(&ink_e2e::alice(), withdraw, 0, None)
            .await
            .expect("Failed to call withdraw");

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_refund(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Deploy the contract
        let constructor = Crowdfunding::new(1000, 100); // Goal: 1000, Deadline: Block 100
        let contract = client
            .instantiate("crowdfunding", &ink_e2e::alice(), constructor)
            .submit()
            .await
            .expect("Failed to instantiate contract");
        let mut call = contract.call::<Crowdfunding>();

        // Alice contributes 500
        let contribute = build_message::<Crowdfunding>(contract.account_id.clone())
            .call(|crowdfunding| crowdfunding.contribute());
        client
            .call(&ink_e2e::alice(), contribute, 500, None)
            .await
            .expect("Failed to call contribute");

        // Fast-forward to after the deadline
        client
            .advance_block()
            .await
            .expect("Failed to advance block");

        // Alice requests a refund
        let refund = build_message::<Crowdfunding>(contract.account_id.clone())
            .call(|crowdfunding| crowdfunding.refund());
        client
            .call(&ink_e2e::alice(), refund, 0, None)
            .await
            .expect("Failed to call refund");

        // Check total raised (should be 0 after refund)
        let total_raised = call.get_total_raised().dry_run().await?;
        assert_eq!(total_raised, 0);

        Ok(())
    }
}