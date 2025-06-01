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

use super::types::{Address, Hash, Id};

#[allow(dead_code)]
pub struct Sign {
    workflow_id: Id,
    inquire_id: Id,
    signer: Address,
    signature_hash: Hash,
    tx_hash: Hash,
    created_at: u64,
}

/// Sign contract interface
pub trait SignContract {
    /// Create signature record
    fn create_sign(
        &self,
        workflow_id: Id,
        inquire_id: Id,
        signer: Address,
        signature_hash: Hash,
    ) -> Id;

    /// Get signature details
    fn get_sign_details(&self, sign_id: Id) -> Sign;

    /// Get signature ID by inquiry ID
    fn get_sign_by_inquire(&self, inquire_id: Id) -> Id;
}
