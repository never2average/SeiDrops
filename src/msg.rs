use crate::error::ContractError;
use crate::state::{Airdrop,InstantiateMsg};

use cosmwasm_std::{
    attr, entry_point, Addr, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use serde_json;


// Define the entry points for the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    args: InstantiateMsg,
) -> Result<Response, ContractError> {
    let amount = args.amount;
    let token_contract = args.token_contract;
    let recipients = args.recipients;
    let denom = args.denom;

    let mut recipients_bytes = Vec::new();
    let mut amount_bytes = Vec::new();
    match serde_json::to_writer(&mut recipients_bytes, &recipients){
        Ok(v) => v,
        Err(_e) => return Err(ContractError::Std(StdError::generic_err("Could not initialize recipients"))),
    };
    match serde_json::to_writer(&mut amount_bytes, &amount){
        Ok(v) => v,
        Err(_e) => return Err(ContractError::Std(StdError::generic_err("Could not initialize amounts to be sent"))),
    };
    deps.storage.set(b"amount", &amount_bytes);
    deps.storage
        .set(b"token_contract", token_contract.as_bytes());
    deps.storage.set(b"recipients", &recipients_bytes);
    deps.storage.set(b"contract_owner", info.sender.as_bytes());
    deps.storage.set(b"denom",denom.as_bytes());

    let mut attributes = vec![attr("action", "instantiate")];
    attributes.push(attr("owner", info.sender.to_string()));

    Ok(Response::new().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    args: InstantiateMsg,
) -> StdResult<Response> {
    let amounts: Vec<u128> = match serde_json::from_slice(&deps.storage.get(b"amount").unwrap()) {
        Ok(v) => v,
        Err(_e) => return Err(StdError::generic_err("Invalid token amounts")),
    };
    let token_contract = deps.api.addr_validate(match &std::string::String::from_utf8(match deps.storage.get(b"token_contract"){
        Some(v) => v,
        None => return Err(StdError::generic_err("Invalid token contract address")),
    }){
        Ok(v) => v,
        Err(_e) => return Err(StdError::generic_err("Invalid token contract address"))
    })?;

    let recipients: Vec<Addr> =
        match serde_json::from_slice(&deps.storage.get(b"recipients").unwrap()) {
            Ok(v) => v,
            Err(_e) => return Err(StdError::generic_err("Invalid recipients address")),
        };

    let mut airdrop = Airdrop::new(amounts, token_contract, recipients);
    airdrop.handle(deps, _env, args, info)
}
