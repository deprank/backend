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

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{
    context::Context,
    handlers::{allocation, contribution, contributor, dependency, project, workflow},
};

pub fn build() -> Router<Arc<Context>> {
    Router::new()
        // projects
        .route("/v1/projects/{owner}/{name}", get(project::get))
        //
        .route("/v1/projects/{owner}/{name}/dependencies", get(dependency::list))
        .route("/v1/projects/{owner}/{name}/dependencies/{dep}", get(dependency::get))
        //
        .route("/v1/projects/{owner}/{name}/contributors", get(contributor::list))
        .route("/v1/projects/{owner}/{name}/contributors/{username}", get(contributor::get))
        //
        // workflows
        .route("/v1/workflows", post(workflow::create))
        .route("/v1/workflows/{id}", delete(workflow::delete))
        .route("/v1/workflows/{id}", get(workflow::get))
        //
        .route("/v1/workflows/{id}/contributions", get(contribution::list))
        .route(
            "/v1/workflows/{workflow_id}/contributions/{contribution_id}",
            get(contribution::get),
        )
        //
        .route("/v1/workflows/{id}/allocations", get(allocation::list))
        .route("/v1/workflows/{workflow_id}/allocations/{allocation_id}", get(allocation::get))
}
