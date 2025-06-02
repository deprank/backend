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

use anyhow::Result;
use std::future::Future;

use super::types::{Address, Hash, Id, Number};

#[allow(dead_code)]
pub struct Allocation {
    workflow_id: Id,
    sign_id: Id,
    recipient: Address,
    amount: Number,
    token_address: Address,
    tx_hash: Hash,
    created_at: u64,
    status: Status,
}

pub enum Status {
    Pending,
    Executed,
    Failed,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "0"),
            Self::Executed => write!(f, "1"),
            Self::Failed => write!(f, "2"),
        }
    }
}

/// Allocation Contract Interface
pub trait AllocationContract {
    /// Create allocation record
    fn create_allocation(
        &self,
        workflow_id: Id,
        sign_id: Id,
        recipient: Address,
        amount: Number,
        token_address: Address,
    ) -> impl Future<Output = Result<Id>>;

    /// Update allocation status
    fn update_allocation_status(
        &self,
        allocation_id: Id,
        status: Status,
    ) -> impl Future<Output = Result<bool>>;

    /// Get allocation details
    fn get_allocation_details(&self, allocation_id: Id)
        -> impl Future<Output = Result<Allocation>>;

    /// Get allocation ID by sign ID
    fn get_allocation_by_sign(&self, sign_id: Id) -> impl Future<Output = Result<Id>>;
}
