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
use serde::{Deserialize, Serialize};
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    core::types::{BlockId, BlockTag, Call, Felt, FunctionCall, InvokeTransactionResult},
    macros::selector,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
    signers::{LocalWallet, SigningKey},
};
use starknet_ff::FieldElement;
use std::str::FromStr;
use tracing::{debug, info};

use crate::contracts::{
    allocation::{Allocation, AllocationContract, Status as AllocationStatus},
    inquire::{Inquire, InquireContract},
    receipt::{Receipt, ReceiptContract, ReceiptMetadata},
    sign::{Sign, SignContract},
    types::*,
    workflow::{Dependency, Step, StepType, Workflow, WorkflowContract},
    Contract,
};

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
pub struct StarkReceiptMetadata {
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

#[derive(Clone, clap::Parser)]
pub struct StarknetConfig {
    /// URL of the Starknet JSON-RPC endpoint
    #[clap(long, env = "STARKNET_RPC_URL")]
    pub starknet_rpc_url: String,

    /// Private key of the Starknet account
    #[clap(long, env = "STARKNET_PRIVATE_KEY")]
    pub starknet_private_key: String,

    /// Address of the Starknet account
    #[clap(long, env = "STARKNET_ACCOUNT_ADDRESS")]
    pub starknet_account_address: String,

    /// Chain ID of the Starknet network
    #[clap(long, env = "STARKNET_CHAIN_ID")]
    pub starknet_chain_id: String,

    /// Address of the Allocation contract
    #[clap(long, env = "ALLOCATION_CONTRACT_ADDRESS")]
    pub allocation_contract_address: String,

    /// Address of the Inquire contract
    #[clap(long, env = "INQUIRE_CONTRACT_ADDRESS")]
    pub inquire_contract_address: String,

    /// Address of the Receipt contract
    #[clap(long, env = "RECEIPT_CONTRACT_ADDRESS")]
    pub receipt_contract_address: String,

    /// Address of the Sign contract
    #[clap(long, env = "SIGN_CONTRACT_ADDRESS")]
    pub sign_contract_address: String,

    /// Address of the Workflow contract
    #[clap(long, env = "WORKFLOW_CONTRACT_ADDRESS")]
    pub workflow_contract_address: String,
}

/// Starknet implementation of the Contract trait
///
/// This struct provides concrete implementations for all contract operations
/// on the Starknet blockchain, including workflow management, allocations,
/// inquiries, receipts, and signatures.
#[allow(dead_code)]
pub struct StarknetContract {
    /// JSON-RPC client for Starknet network
    provider: JsonRpcClient<HttpTransport>,

    /// Starknet account with signing capability
    account: SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,

    /// Address of the Allocation contract
    allocation_contract_address: Felt,

    /// Address of the Inquire contract
    inquire_contract_address: Felt,

    /// Address of the Receipt contract
    receipt_contract_address: Felt,

    /// Address of the Sign contract
    sign_contract_address: Felt,

    /// Address of the Workflow contract
    workflow_contract_address: Felt,
}

impl StarknetContract {
    pub fn new(config: &StarknetConfig) -> Self {
        // Create provider used to access to the Starknet network.
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&config.starknet_rpc_url).expect("Invalid Starknet RPC URL format"),
        ));

        // Create account object.
        let key_pair = SigningKey::from_secret_scalar(
            Felt::from_hex(&config.starknet_private_key).expect("Invalid Starknet private key"),
        );
        let signer = LocalWallet::from_signing_key(key_pair);
        let account_address = Felt::from_hex(&config.starknet_account_address)
            .expect("Invalid Starknet account address");
        let chain_id =
            Felt::from_str(&config.starknet_chain_id).expect("Invalid Starknet chain id");

        let account = SingleOwnerAccount::new(
            provider.clone(),
            signer,
            account_address,
            chain_id,
            ExecutionEncoding::New,
        );

        // parse contract addresses.
        let allocation_contract_address = Felt::from_hex(&config.allocation_contract_address)
            .expect("Invalid allocation contract address");

        let inquire_contract_address = Felt::from_hex(&config.inquire_contract_address)
            .expect("Invalid inquire contract address");

        let receipt_contract_address = Felt::from_hex(&config.receipt_contract_address)
            .expect("Invalid receipt contract address");

        let sign_contract_address =
            Felt::from_hex(&config.sign_contract_address).expect("Invalid sign contract address");

        let workflow_contract_address = Felt::from_hex(&config.workflow_contract_address)
            .expect("Invalid workflow contract address");

        Self {
            provider,
            account,
            allocation_contract_address,
            inquire_contract_address,
            receipt_contract_address,
            sign_contract_address,
            workflow_contract_address,
        }
    }

    #[allow(dead_code)]
    /// Call contract function (read-only operation)
    async fn call(
        &self,
        contract_address: &Felt,
        selector: &Felt,
        calldata: Vec<Felt>,
    ) -> Result<Vec<Felt>> {
        let function_call = FunctionCall {
            contract_address: *contract_address,
            entry_point_selector: *selector,
            calldata,
        };

        info!("Attempting contract call (read-only operation)...");

        match self.provider.call(function_call, BlockId::Tag(BlockTag::Latest)).await {
            Ok(result) => {
                info!("Call successful! Result: {:?}", result);
                Ok(result)
            }
            Err(e) => Err(anyhow!("Contract call failed: {:?}", e)),
        }
    }

    /// Execute transaction
    async fn execute(
        &self,
        contract_address: &Felt,
        selector: &Felt,
        calldata: Vec<Felt>,
    ) -> Result<InvokeTransactionResult> {
        debug!(
            "Execute transaction, contract_address: {}, selector: {}, calldata: {:?}",
            contract_address, selector, calldata
        );

        // Create function call object
        let calls = vec![Call { to: *contract_address, selector: *selector, calldata }];

        // Execute transaction
        let result = self.account.execute_v3(calls).send().await?;
        info!("Transaction sent! Transaction hash: 0x{:x}", result.transaction_hash);

        // Print Starkscan link
        info!("Transaction submitted to network. View transaction status on Starkscan:");
        info!("https://sepolia.starkscan.co/tx/0x{:x}", result.transaction_hash);

        Ok(result)
    }
}

impl Contract for StarknetContract {
    fn chain() -> &'static str {
        "Starknet"
    }
}

impl AllocationContract for StarknetContract {
    fn create_allocation(
        &self,
        _workflow_id: Id,
        _sign_id: Id,
        _recipient: Address,
        _amount: Number,
        _token_address: Address,
    ) -> Id {
        todo!()
    }

    fn update_allocation_status(&self, _allocation_id: Id, _status: AllocationStatus) -> bool {
        todo!()
    }

    fn get_allocation_details(&self, _allocation_id: Id) -> Allocation {
        todo!()
    }

    fn get_allocation_by_sign(&self, _sign_id: Id) -> Id {
        todo!()
    }
}

impl InquireContract for StarknetContract {
    fn create_inquire(
        &self,
        _workflow_id: Id,
        _inquirer: Address,
        _inquiree: Address,
        _question: String,
    ) -> Id {
        todo!()
    }

    fn respond_to_inquire(&self, _inquire_id: Id, _response: String) -> bool {
        todo!()
    }

    fn reject_inquire(&self, _inquire_id: Id) -> bool {
        todo!()
    }

    fn get_inquire_details(&self, _inquire_id: Id) -> Inquire {
        todo!()
    }
}

impl ReceiptContract for StarknetContract {
    fn create_receipt(
        &self,
        _workflow_id: Id,
        _dependency_url: String,
        _metadata: ReceiptMetadata,
        _metadata_hash: Hash,
        _metadata_uri: Hash,
    ) -> Id {
        todo!()
    }

    fn get_receipt_details(&self, _receipt_id: Id) -> (Receipt, ReceiptMetadata) {
        todo!()
    }

    fn verify_metadata(&self, _receipt_id: Id, _provided_hash: Hash) -> bool {
        todo!()
    }

    fn update_tx_hash(&self, _receipt_id: Id, _tx_hash: Hash) {
        todo!()
    }
}

impl SignContract for StarknetContract {
    fn create_sign(
        &self,
        _workflow_id: Id,
        _inquire_id: Id,
        _signer: Address,
        _signature_hash: Hash,
    ) -> Id {
        todo!()
    }

    fn get_sign_details(&self, _sign_id: Id) -> Sign {
        todo!()
    }

    fn get_sign_by_inquire(&self, _inquire_id: Id) -> Id {
        todo!()
    }
}

impl WorkflowContract for StarknetContract {
    async fn create_workflow(&self, github_owner: Owner, wallet_address: Address) -> Result<Id> {
        info!("Starting workflow creation");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let wallet_address = Felt::from_hex(&wallet_address).expect("Invalid wallet address");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("create_workflow"),
                vec![github_owner, wallet_address],
            )
            .await?;

        Ok(Id::new())
    }

    async fn create_dependency(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        name: String,
        repository_url: String,
        license: String,
        metadata_json: String,
    ) -> Result<Id> {
        info!("Starting dependency creation");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let name = Felt::from_str(&name).expect("Invalid name");
        let repository_url = Felt::from_str(&repository_url).expect("Invalid repository url");
        let license = Felt::from_str(&license).expect("Invalid license");
        let metadata_json = Felt::from_str(&metadata_json).expect("Invalid metadata json");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("create_dependency"),
                vec![github_owner, workflow_id, name, repository_url, license, metadata_json],
            )
            .await?;

        Ok(Id::new())
    }

    async fn add_step(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
        step_type: StepType,
        tx_hash: Hash,
        related_entity_id: Id,
    ) -> Result<Id> {
        info!("Starting add step");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let dependency_idx = Felt::from_str(&dependency_idx).expect("Invalid dependency index");
        let step_type = Felt::from_str(&step_type.to_string()).expect("Invalid step type");
        let tx_hash = Felt::from_str(&tx_hash).expect("Invalid transaction hash");
        let related_entity_id =
            Felt::from_str(&related_entity_id).expect("Invalid related entity id");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("add_step"),
                vec![
                    github_owner,
                    workflow_id,
                    dependency_idx,
                    step_type,
                    tx_hash,
                    related_entity_id,
                ],
            )
            .await?;

        Ok(Id::new())
    }

    async fn finish_dependency(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<bool> {
        info!("Starting finish dependency");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let dependency_idx = Felt::from_str(&dependency_idx).expect("Invalid dependency index");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("finish_dependency"),
                vec![github_owner, workflow_id, dependency_idx],
            )
            .await?;

        Ok(true)
    }

    async fn finish_workflow(&self, github_owner: Owner, workflow_id: Id) -> Result<bool> {
        info!("Starting finish workflow");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("finish_workflow"),
                vec![github_owner, workflow_id],
            )
            .await?;

        Ok(true)
    }

    async fn get_workflow_status(&self, github_owner: Owner, workflow_id: Id) -> Result<Workflow> {
        info!("Starting get workflow status");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");

        let result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_workflow_status"),
                vec![github_owner, workflow_id],
            )
            .await?;

        let _workflow = result.first().ok_or(anyhow!("Not found workflow"))?;
        todo!()
    }

    async fn get_dependencies(
        &self,
        github_owner: Owner,
        workflow_id: Id,
    ) -> Result<Vec<Dependency>> {
        info!("Starting get dependencies");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");

        let _result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_dependencies"),
                vec![github_owner, workflow_id],
            )
            .await?;

        todo!()
    }

    async fn get_steps(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<Vec<Step>> {
        info!("Starting get steps");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let dependency_idx = Felt::from_str(&dependency_idx).expect("Invalid dependency index");

        let _result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_steps"),
                vec![github_owner, workflow_id, dependency_idx],
            )
            .await?;

        todo!()
    }

    async fn get_step_by_tx_hash(&self, tx_hash: Hash) -> Result<Option<(Owner, Id, Id, Id)>> {
        info!("Starting get step by tx hash");

        let tx_hash = Felt::from_hex(&tx_hash).expect("Invalid transaction hash");

        let _result = self
            .call(&self.workflow_contract_address, &selector!("get_step_by_tx_hash"), vec![tx_hash])
            .await?;

        todo!()
    }

    async fn get_complete_transaction_chain(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<Vec<Hash>> {
        info!("Starting get complete transaction chain");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let dependency_idx = Felt::from_str(&dependency_idx).expect("Invalid dependency index");

        let _result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_complete_transaction_chain"),
                vec![github_owner, workflow_id, dependency_idx],
            )
            .await?;

        todo!()
    }

    async fn get_workflow_count(&self, github_owner: Owner) -> Result<Number> {
        info!("Starting get workflow count");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");

        let result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_workflow_count"),
                vec![github_owner],
            )
            .await?;

        let count = result.first().unwrap_or(&Felt::ZERO);
        Ok(count.to_string())
    }

    async fn get_all_workflows(&self, github_owner: Owner) -> Result<Vec<(Number, Workflow)>> {
        info!("Starting get all workflows");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");

        let _result = self
            .call(
                &self.workflow_contract_address,
                &selector!("get_all_workflows"),
                vec![github_owner],
            )
            .await?;

        todo!()
    }

    async fn bind_wallet_address(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        wallet_address: Address,
    ) -> Result<bool> {
        info!("Starting bind wallet address");

        let github_owner = Felt::from_str(&github_owner).expect("Invalid GitHub username");
        let workflow_id = Felt::from_str(&workflow_id).expect("Invalid workflow id");
        let wallet_address = Felt::from_hex(&wallet_address).expect("Invalid wallet address");

        let _ = self
            .execute(
                &self.workflow_contract_address,
                &selector!("bind_wallet_address"),
                vec![github_owner, workflow_id, wallet_address],
            )
            .await?;

        Ok(true)
    }

    fn unbind_wallet_address(&self, _github_owner: Owner, _workflow_id: Id) -> bool {
        todo!()
    }

    fn change_wallet_address(
        &self,
        _github_owner: Owner,
        _workflow_id: Id,
        _new_wallet_address: Address,
    ) -> bool {
        todo!()
    }
}
