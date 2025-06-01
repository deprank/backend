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

use super::types::{Hash, Id};

#[allow(dead_code)]
pub struct Receipt {
    workflow_id: Id,
    dependency_url: String,
    tx_hash: Hash,
    created_at: u64,
    /// Hash value of the complete JSON
    metadata_hash: Hash,
    /// URI pointing to the complete JSON
    metadata_uri: String,
}

/// Common key fields, stored directly on the chain
pub struct ReceiptMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub license: String,
}

/// Receipt contract interface
pub trait ReceiptContract {
    /// Create receipt and store metadata
    fn create_receipt(
        &self,
        workflow_id: Id,
        dependency_url: String,
        metadata: ReceiptMetadata,
        metadata_hash: Hash,
        metadata_uri: Hash,
    ) -> Id;

    /// Get receipt details
    fn get_receipt_details(&self, receipt_id: Id) -> (Receipt, ReceiptMetadata);

    /// Verify metadata
    fn verify_metadata(&self, receipt_id: Id, provided_hash: Hash) -> bool;

    /// Update transaction hash
    fn update_tx_hash(&self, receipt_id: Id, tx_hash: Hash);
}
