use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AirdropMsg {
    AddRecipient {
        recipients: Vec<Addr>,
        amounts: Vec<u128>,
    },
    ClaimAirdrop {
        claimer: Addr,
        amount: u128,
    },
}

// Define the struct to hold the state of the contract
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Airdrop {
    pub amount: Vec<u128>,
    pub token_contract: Addr,
    pub recipients: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub amount: Vec<u128>,
    pub token_contract: Addr,
    pub recipients: Vec<Addr>,
    pub denom: String,
    pub message_type: AirdropMsg,
}
