## Stellar Crowdfunding Project
A decentralized crowdfunding smart contract built using the ink! smart contract framework for the Stellar blockchain.

## Overview
This project is a decentralized crowdfunding platform implemented as a smart contract using the ink! framework. It allows users to create campaigns, contribute funds, and withdraw funds if the campaign goal is met. If the goal is not met by the deadline, contributors can request refunds.

## Features
#### 1.Create Campaigns: Users can create crowdfunding campaigns with a specific goal and deadline.

#### 2.Contribute Funds: Users can contribute funds to active campaigns.

#### 3. Withdraw Funds: Campaign creators can withdraw funds if the goal is met.

#### 4. Refund Contributions: If the campaign goal is not met, contributors can request refunds.

#### 5. Event Logging: All major actions (e.g., contributions, withdrawals, refunds) are logged as events.

## Getting Started
### Prerequisites
Rust (latest stable version)

cargo-contract (for building and deploying ink! contracts)

Node.js (for testing with ink! E2E framework)

## Installation
#### 1. Clone the repository:
git clone https://github.com/HD-Devcamp-Africa/Cy_Crowdfunding.git
cd Cy_Crowdfunding

#### 2. Install dependencies:
cargo install cargo-contract --force

#### 3. Build the contract:
cargo +nightly contract build

## License
This project is licensed under the MIT License. 

## Acknowledgments
ink! for the smart contract framework.
Substrate for the blockchain framework.
HD Devcamp Africa for the opportunity to work on this project.
