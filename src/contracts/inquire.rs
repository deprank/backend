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

use super::types::{Address, Id};

#[allow(dead_code)]
pub struct Inquire {
    workflow_id: Id,
    inquirer: Address,
    inquiree: Address,
    question: String,
    response: String,
    status: Status,
    created_at: u64,
    responded_at: u64,
}

pub enum Status {
    Pending,
    Responded,
    Rejected,
}

/// Inquire contract interface
pub trait InquireContract {
    /// Create inquiry
    fn create_inquire(
        &self,
        workflow_id: Id,
        inquirer: Address,
        inquiree: Address,
        question: String,
    ) -> Id;

    /// Respond to inquiry
    fn respond_to_inquire(&self, inquire_id: Id, response: String) -> bool;

    /// Reject inquiry
    fn reject_inquire(&self, inquire_id: Id) -> bool;

    /// Get inquiry details
    fn get_inquire_details(&self, inquire_id: Id) -> Inquire;
}
