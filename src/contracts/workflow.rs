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

use std::future::Future;

use anyhow::Result;

use super::types::{Address, Hash, Id, Number, Owner};

#[allow(dead_code)]
pub struct Workflow {
    owner: Owner,
    /// Associated multisig wallet address
    wallet_address: Address,
    status: Status,
    created_at: u64,
    last_updated_at: u64,
}

#[allow(dead_code)]
pub struct Dependency {
    /// Dependency name or ID
    name: String,
    repository_url: String,
    license: String,
    /// JSON formatted additional data
    metadata_json: String,
    status: Status,
    created_at: u64,
    last_updated_at: u64,
}

#[allow(dead_code)]
pub struct Step {
    step_type: StepType,
    tx_hash: Hash,
    // Related entity ID (receipt_id, inquire_id, etc.)
    related_entity_id: Id,
    timestamp: u64,
    /// Previous step index, used for linking
    prev_step_index: Id,
}

pub enum StepType {
    Receipt,
    Inquire,
    Sign,
    Allocation,
}

impl std::fmt::Display for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::Receipt => write!(f, "1"),
            StepType::Inquire => write!(f, "2"),
            StepType::Sign => write!(f, "3"),
            StepType::Allocation => write!(f, "4"),
        }
    }
}

pub enum Status {
    Created,
    InProgress,
    Completed,
}

/// Workflow contract interface
pub trait WorkflowContract {
    /// Create workflow
    fn create_workflow(
        &self,
        github_owner: Owner,
        wallet_address: Address,
    ) -> impl Future<Output = Result<Id>>;

    /// Create dependency
    fn create_dependency(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        name: String,
        repository_url: String,
        license: String,
        metadata_json: String,
    ) -> impl Future<Output = Result<Id>>;

    /// Add step
    fn add_step(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
        step_type: StepType,
        tx_hash: Hash,
        related_entity_id: Id,
    ) -> impl Future<Output = Result<Id>>;

    /// Complete dependency
    fn finish_dependency(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> impl Future<Output = Result<bool>>;

    /// Complete workflow
    fn finish_workflow(
        &self,
        github_owner: Owner,
        workflow_id: Id,
    ) -> impl Future<Output = Result<bool>>;

    /// Get workflow status
    fn get_workflow_status(&self, github_owner: Owner, workflow_id: Id) -> Workflow;

    /// Get workflow dependencies
    fn get_dependencies(&self, github_owner: Owner, workflow_id: Id) -> Vec<Dependency>;

    /// Get dependency steps
    fn get_steps(&self, github_owner: Owner, workflow_id: Id, dependency_idx: Id) -> Vec<Step>;

    /// Get step by transaction hash
    /// (github_owner, workflow_id, dependency_index, step_index)
    fn get_step_by_tx_hash(&self, tx_hash: Hash) -> Option<(Owner, Id, Id, Id)>;

    /// Get complete transaction chain
    fn get_complete_transaction_chain(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        dependency_idx: Id,
    ) -> Vec<Hash>;

    /// Get user workflow count
    fn get_workflow_count(&self, github_owner: Owner) -> Number;

    /// Get all user workflows
    fn get_all_workflows(&self, github_owner: Owner) -> Vec<(Number, Workflow)>;

    /// Bind multisig wallet address to workflow
    fn bind_wallet_address(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        wallet_address: Address,
    ) -> bool;

    /// Unbind multisig wallet address
    fn unbind_wallet_address(&self, github_owner: Owner, workflow_id: Id) -> bool;

    /// Change multisig wallet address
    fn change_wallet_address(
        &self,
        github_owner: Owner,
        workflow_id: Id,
        new_wallet_address: Address,
    ) -> bool;
}
