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

use crate::{
    config::Config,
    contracts::{
        allocation::{Allocation, AllocationContract, Status as AllocationStatus},
        impls::starknet::StarknetContract,
        inquire::{Inquire, InquireContract},
        receipt::{Receipt, ReceiptContract, ReceiptMetadata},
        sign::{Sign, SignContract},
        types::*,
        workflow::{Dependency, Step, StepType, Workflow, WorkflowContract},
        Contract,
    },
};

use anyhow::Result;

/// A service that provides contract operations by wrapping a Starknet contract implementation.
///
/// This struct acts as a facade to the underlying Starknet contract, providing methods
/// for various contract operations like allocation, inquiry, receipt, signing, and workflow
/// management. It implements multiple contract traits to provide a unified interface for all
/// contract operations.
pub struct ContractService {
    instance: StarknetContract,
}

impl ContractService {
    pub fn new(config: &Config) -> Self {
        Self { instance: StarknetContract::new(&config.starknet_config) }
    }
}

impl Contract for ContractService {
    fn chain() -> &'static str {
        StarknetContract::chain()
    }
}

impl AllocationContract for ContractService {
    fn create_allocation(
        &self,
        workflow_id: Id,
        sign_id: Id,
        recipient: Address,
        amount: Number,
        token_address: Address,
    ) -> Id {
        self.instance.create_allocation(workflow_id, sign_id, recipient, amount, token_address)
    }

    fn update_allocation_status(&self, allocation_id: Id, status: AllocationStatus) -> bool {
        self.instance.update_allocation_status(allocation_id, status)
    }

    fn get_allocation_details(&self, allocation_id: Id) -> Allocation {
        self.instance.get_allocation_details(allocation_id)
    }

    fn get_allocation_by_sign(&self, sign_id: Id) -> Id {
        self.instance.get_allocation_by_sign(sign_id)
    }
}

impl InquireContract for ContractService {
    fn create_inquire(
        &self,
        workflow_id: Id,
        inquirer: Address,
        inquiree: Address,
        question: String,
    ) -> Id {
        self.instance.create_inquire(workflow_id, inquirer, inquiree, question)
    }

    fn respond_to_inquire(&self, inquire_id: Id, response: String) -> bool {
        self.instance.respond_to_inquire(inquire_id, response)
    }

    fn reject_inquire(&self, inquire_id: Id) -> bool {
        self.instance.reject_inquire(inquire_id)
    }

    fn get_inquire_details(&self, inquire_id: Id) -> Inquire {
        self.instance.get_inquire_details(inquire_id)
    }
}

impl ReceiptContract for ContractService {
    fn create_receipt(
        &self,
        workflow_id: Id,
        dependency_url: String,
        metadata: ReceiptMetadata,
        metadata_hash: Hash,
        metadata_uri: Hash,
    ) -> Id {
        self.instance.create_receipt(
            workflow_id,
            dependency_url,
            metadata,
            metadata_hash,
            metadata_uri,
        )
    }

    fn get_receipt_details(&self, receipt_id: Id) -> (Receipt, ReceiptMetadata) {
        self.instance.get_receipt_details(receipt_id)
    }

    fn verify_metadata(&self, receipt_id: Id, provided_hash: Hash) -> bool {
        self.instance.verify_metadata(receipt_id, provided_hash)
    }

    fn update_tx_hash(&self, receipt_id: Id, tx_hash: Hash) {
        self.instance.update_tx_hash(receipt_id, tx_hash);
    }
}

impl SignContract for ContractService {
    fn create_sign(
        &self,
        workflow_id: Id,
        inquire_id: Id,
        signer: Address,
        signature_hash: Hash,
    ) -> Id {
        self.instance.create_sign(workflow_id, inquire_id, signer, signature_hash)
    }

    fn get_sign_details(&self, sign_id: Id) -> Sign {
        self.instance.get_sign_details(sign_id)
    }

    fn get_sign_by_inquire(&self, inquire_id: Id) -> Id {
        self.instance.get_sign_by_inquire(inquire_id)
    }
}

impl WorkflowContract for ContractService {
    async fn create_workflow(&self, github_owner: Owner, wallet_address: Address) -> Result<Id> {
        self.instance.create_workflow(github_owner, wallet_address).await
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
        self.instance
            .create_dependency(
                github_owner,
                workflow_id,
                name,
                repository_url,
                license,
                metadata_json,
            )
            .await
    }

    async fn add_step(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_index: Id,
        step_type: StepType,
        tx_hash: Hash,
        related_entity_id: Id,
    ) -> Result<Id> {
        self.instance
            .add_step(
                github_owner,
                workflow_id,
                dependency_index,
                step_type,
                tx_hash,
                related_entity_id,
            )
            .await
    }

    async fn finish_dependency(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<bool> {
        self.instance.finish_dependency(github_owner, workflow_id, dependency_idx).await
    }

    async fn finish_workflow(&self, github_owner: Owner, workflow_id: Id) -> Result<bool> {
        self.instance.finish_workflow(github_owner, workflow_id).await
    }

    async fn get_workflow_status(&self, github_owner: Owner, workflow_id: Id) -> Result<Workflow> {
        self.instance.get_workflow_status(github_owner, workflow_id).await
    }

    async fn get_dependencies(
        &self,
        github_owner: Owner,
        workflow_id: Id,
    ) -> Result<Vec<Dependency>> {
        self.instance.get_dependencies(github_owner, workflow_id).await
    }

    async fn get_steps(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<Vec<Step>> {
        self.instance.get_steps(github_owner, workflow_id, dependency_idx).await
    }

    async fn get_step_by_tx_hash(&self, tx_hash: Hash) -> Result<Option<(Owner, Id, Id, Id)>> {
        self.instance.get_step_by_tx_hash(tx_hash).await
    }

    async fn get_complete_transaction_chain(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Result<Vec<Hash>> {
        self.instance
            .get_complete_transaction_chain(github_owner, workflow_id, dependency_idx)
            .await
    }

    async fn get_workflow_count(&self, github_owner: Owner) -> Result<Number> {
        self.instance.get_workflow_count(github_owner).await
    }

    async fn get_all_workflows(&self, github_owner: Owner) -> Result<Vec<(Number, Workflow)>> {
        self.instance.get_all_workflows(github_owner).await
    }

    async fn bind_wallet_address(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        wallet_address: Address,
    ) -> Result<bool> {
        self.instance.bind_wallet_address(github_owner, workflow_id, wallet_address).await
    }

    async fn unbind_wallet_address(&self, github_owner: Owner, workflow_id: Id) -> Result<bool> {
        self.instance.unbind_wallet_address(github_owner, workflow_id).await
    }

    fn change_wallet_address(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        new_wallet_address: Address,
    ) -> bool {
        self.instance.change_wallet_address(github_owner, workflow_id, new_wallet_address)
    }
}
