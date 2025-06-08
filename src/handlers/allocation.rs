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

//! The Allocation Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{context::Context, errors::Result};

/// Get allocations list of the workflow
#[utoipa::path(
    operation_id = "get-allocations-list",
    get, path = "/v1/workflows/{id}/allocations",
    params(
        ("id" = Uuid, description = "The id of workflow"),
    ),
    responses(
        (status = 200, description = "Allocations retrieved successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to get workflow")
    ),
    tag = "Allocation"
)]
pub async fn list(
    State(_ctx): State<Arc<Context>>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    Ok(Vec::new())
}

/// Get the allocation detail of the workflow
#[utoipa::path(
    operation_id = "get-allocation-detail",
    get, path = "/v1/workflows/{id}/allocations/{allocation_id}",
    params(
        ("id" = Uuid, description = "The id of workflow"),
        ("allocation_id" = Uuid, description = "The id of allocation"),
    ),
    responses(
        (status = 200, description = "Allocation retrieved successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to get workflow")
    ),
    tag = "Allocation"
)]
pub async fn get(
    State(_ctx): State<Arc<Context>>,
    Path((_id, _allocation_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    Ok(Vec::new())
}
