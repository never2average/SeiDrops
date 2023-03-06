use crate::state::{Airdrop, AirdropMsg, InstantiateMsg};
use cosmwasm_std::coins;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use log::info;

const CONTRACT_NAME: &str = "crates.io:web3-crm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION"); // 0.1.0

// Define the implementation of the contract
impl Airdrop {
    pub fn new(amount: Vec<u128>, token_contract: Addr, recipients: Vec<Addr>) -> Self {
        Self {
            amount,
            token_contract,
            recipients,
        }
    }

    pub fn handle(
        &mut self,
        deps: DepsMut,
        env: Env,
        msg: InstantiateMsg,
        info: MessageInfo,
    ) -> StdResult<Response> {
        // Only the contract owner is allowed to acess any of these functioms
        let owner =
            match std::string::String::from_utf8(deps.storage.get(b"contract_owner").unwrap()) {
                Ok(v) => v,
                Err(_e) => return Err(StdError::generic_err("Invalid contract owner address")),
            };
        if owner != info.sender.to_string() {
            return Err(StdError::generic_err("Unauthored to acess this"));
        }
        match msg.message_type {
            AirdropMsg::AddRecipient {
                recipients,
                amounts,
            } => {
                self.add_recipients(deps, recipients, amounts, env)?;
            }
            AirdropMsg::ClaimAirdrop { claimer, amount } => {
                self.try_airdrop(deps, env, claimer, amount)?;
            }
        }

        Ok(Response::default())
    }

    fn add_recipients(
        &mut self,
        deps: DepsMut,
        recipients: Vec<Addr>,
        amounts: Vec<u128>,
        _env: Env,
    ) -> StdResult<()> {
        let store = deps.storage;
        let unmutable_recipients = self.recipients.clone();
        for (i, recipient) in recipients.iter().enumerate() {
            let mut exists = false;
            for (j, existing_recipient) in unmutable_recipients.iter().enumerate() {
                if recipient == existing_recipient {
                    self.amount[j] = self.amount[j] + amounts[i];
                    exists = true;
                }
            if exists == false {
                self.recipients.push(recipient.clone());
                self.amount.push(amounts[i].clone());
            }
            }
        }
        store.set(
            b"recipients",
            match &serde_json::to_vec(&self.recipients) {
                Ok(v) => v,
                Err(_e) => return Err(StdError::generic_err("Error in setting recipients")),
            },
        );
        store.set(
            b"amount",
            match &serde_json::to_vec(&self.amount) {
                Ok(v) => v,
                Err(_e) => return Err(StdError::generic_err("Error in setting amount")),
            },
        );
        info!("Added recipients");
        Ok(())
    }

    fn try_airdrop(
        &mut self,
        deps: DepsMut,
        env: Env,
        claimer: Addr,
        amount: u128,
    ) -> StdResult<Response> {
        let mut value = match deps.storage.get(b"token_contract") {
            Some(v) => v,
            None => return Err(StdError::generic_err("Error in fetching contract address")),
        };
        let token_contract =
            deps.api
                .addr_validate(match &std::string::String::from_utf8((&value).to_vec()) {
                    Ok(v) => v,
                    Err(_e) => {
                        return Err(StdError::generic_err(
                            "Error in fetching token contract address",
                        ))
                    }
                });

        value = deps.storage.get(b"denom").unwrap();
        let denom: String = match std::string::String::from_utf8(value) {
            Ok(v) => v,
            Err(_e) => return Err(StdError::generic_err("Error in fetching coin denom")),
        };

        let contract_balance = deps
            .querier
            .query_balance(env.contract.address, "uusd".to_string())
            .unwrap()
            .amount;

        if contract_balance < amount.into() {
            return Err(StdError::generic_err("Insufficient contract balance"));
        };
        for (i, recipient) in self.recipients.iter().enumerate() {
            if claimer == *recipient && self.amount[i] <= amount.into() {
                let msg = Cw20ExecuteMsg::Transfer {
                    recipient: claimer.to_string().clone(),
                    amount: Uint128::from(amount),
                };

                let execute = WasmMsg::Execute {
                    contract_addr: token_contract?.clone().as_str().to_string(),
                    msg: to_binary(&msg).unwrap(),
                    funds: coins(amount, denom),
                };

                CosmosMsg::<cosmwasm_std::CosmosMsg>::Wasm(execute);

                let response = Response::new().add_attribute("method", "try_airdrop");

                return Ok(response);
            }
        }
        return Err(StdError::generic_err(
            "Recipient not found or not enough is owned to him/her as claimed",
        ));
    }
}
