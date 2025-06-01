// Copyright (c) The DepRank Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::{
        chain_id,
        types::{BlockId, BlockTag, Call, Felt, FunctionCall},
        utils::cairo_short_string_to_felt,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
    signers::{LocalWallet, SigningKey},
};
use starknet_ff::FieldElement;
use std::{env, sync::Arc};
use tracing::{error, info};

// Global provider
static PROVIDER: Lazy<Arc<JsonRpcClient<HttpTransport>>> = Lazy::new(|| {
    let rpc_url =
        env::var("STARKNET_RPC_URL").expect("STARKNET_RPC_URL environment variable must be set");
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse(&rpc_url).expect("Invalid RPC URL format"),
    ));
    Arc::new(provider)
});

// Struct definitions corresponding to contract structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDetails {
    pub owner: FieldElement,
    pub wallet_address: FieldElement,
    pub status: FieldElement,
    pub created_at: u64,
    pub last_updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDetails {
    pub name: FieldElement,
    pub repository_url: FieldElement,
    pub license: FieldElement,
    pub metadata_json: FieldElement,
    pub status: FieldElement,
    pub created_at: u64,
    pub last_updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDetails {
    pub step_type: FieldElement,
    pub tx_hash: FieldElement,
    pub related_entity_id: FieldElement,
    pub timestamp: u64,
    pub prev_step_index: FieldElement,
}

// Allocation related struct definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDetails {
    pub workflow_id: FieldElement,
    pub sign_id: FieldElement,
    pub recipient: FieldElement,
    pub amount: FieldElement,
    pub token_address: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
    pub status: FieldElement, // 0: pending, 1: executed, 2: failed
}

// Inquire related struct definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquireDetails {
    pub workflow_id: FieldElement,
    pub inquirer: FieldElement,
    pub inquiree: FieldElement,
    pub question: FieldElement,
    pub response: FieldElement,
    pub status: FieldElement, // 0: pending, 1: responded, 2: rejected
    pub created_at: u64,
    pub responded_at: u64,
}

// Receipt related struct definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptDetails {
    pub workflow_id: FieldElement,
    pub dependency_url: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
    pub metadata_hash: FieldElement,
    pub metadata_uri: FieldElement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptMetadata {
    pub name: FieldElement,
    pub version: FieldElement,
    pub author: FieldElement,
    pub license: FieldElement,
}

// Sign related struct definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignDetails {
    pub workflow_id: FieldElement,
    pub inquire_id: FieldElement,
    pub signer: FieldElement,
    pub signature_hash: FieldElement,
    pub tx_hash: FieldElement,
    pub created_at: u64,
}

/// Public method to get account
pub async fn get_account(
) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, anyhow::Error> {
    // Get private key from environment variable
    let private_key = env::var("STARKNET_PRIVATE_KEY")
        .expect("STARKNET_PRIVATE_KEY environment variable must be set");
    info!("Private key read from environment variable");

    // Set up wallet
    let key_pair =
        SigningKey::from_secret_scalar(Felt::from_hex(&private_key).expect("Invalid private key"));
    let signer = LocalWallet::from_signing_key(key_pair);

    // Get account address from environment variable
    let account_address_str = env::var("STARKNET_ACCOUNT_ADDRESS")
        .expect("STARKNET_ACCOUNT_ADDRESS environment variable must be set");
    info!("Account address read from environment variable");

    let account_address = Felt::from_hex(&account_address_str).expect("Invalid account address");
    let chain_id = chain_id::SEPOLIA; // Using predefined Sepolia chain ID

    // Create account object
    let account = SingleOwnerAccount::new(
        PROVIDER.as_ref().clone(),
        signer,
        account_address,
        chain_id,
        ExecutionEncoding::New,
    );

    Ok(account)
}

/// Public method for contract calls
pub async fn call_contract_function(
    contract_address: Felt,
    selector: Felt,
    calldata: Vec<Felt>,
) -> Result<Vec<Felt>, anyhow::Error> {
    // Create function call object
    let function_call = FunctionCall { contract_address, entry_point_selector: selector, calldata };

    info!("Attempting contract call (read-only operation)...");

    match PROVIDER.as_ref().call(function_call, BlockId::Tag(BlockTag::Latest)).await {
        Ok(result) => {
            info!("Call successful! Result: {:?}", result);
            Ok(result)
        }
        Err(e) => {
            error!("Call failed: {:?}", e);
            error!("This may indicate incorrect parameter format or non-existent function, please check before attempting to send transaction");
            Err(anyhow!("Contract call failed: {:?}", e))
        }
    }
}

/// Create workflow
pub async fn create_workflow(
    github_owner_str: &str,
    wallet_address_str: &str,
) -> Result<(), anyhow::Error> {
    info!(
        "Starting workflow creation, github_owner: {}, wallet_address: {}",
        github_owner_str, wallet_address_str
    );

    // Get account
    let account = get_account().await?;

    // Get contract address from environment variable
    let contract_address_str = env::var("WORKFLOW_CONTRACT_ADDRESS")
        .expect("WORKFLOW_CONTRACT_ADDRESS environment variable must be set");
    info!("Contract address read from environment variable: {}", contract_address_str);

    let contract_address = Felt::from_hex(&contract_address_str).expect("Invalid contract address");

    // Convert string to felt, ensuring proper encoding
    let github_owner =
        cairo_short_string_to_felt(github_owner_str).expect("Invalid GitHub username");
    info!("Converted github_owner: {:?}", github_owner);

    // Process wallet address parameter
    let wallet_address = Felt::from_hex(wallet_address_str).expect("Invalid wallet address");
    info!("Wallet address: {:?}", wallet_address);

    // Use correct function selector
    let function_selector =
        Felt::from_hex("0x5911913ce5ab907c3a2d99993ea1a79752241ca82352c7962c5c228d183b6e")
            .expect("Invalid selector");

    // Prepare call parameters
    let calldata = vec![github_owner, wallet_address];

    // First try read-only call to validate function
    let _result =
        match call_contract_function(contract_address, function_selector, calldata.clone()).await {
            Ok(result) => result,
            Err(e) => {
                info!("Read-only call validation failed, aborting transaction");
                return Err(e);
            }
        };

    // Create function call object
    let calls = vec![Call { to: contract_address, selector: function_selector, calldata }];

    // Execute transaction
    info!("Sending create_workflow transaction...");
    let tx_result = account.execute_v3(calls).send().await?;
    info!("Transaction sent! Transaction hash: 0x{:x}", tx_result.transaction_hash);

    // Print Starkscan link
    info!("Transaction submitted to network. View transaction status on Starkscan:");
    info!("https://sepolia.starkscan.co/tx/0x{:x}", tx_result.transaction_hash);

    Ok(())
}
