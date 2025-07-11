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

//! The Workflow Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    context::Context, errors::Result, requests::workflow::CreateWorkflowRequest,
    responses::workflow::WorkflowResponse, services::workflow::WorkflowService,
};

/// Create a workflow in the current account.
#[utoipa::path(
    operation_id = "create-workflow",
    post, path = "/v1/workflows",
    request_body(
        content = inline(CreateWorkflowRequest),
        description = "Create workflow request",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Workflow created successfully", body = WorkflowResponse)
    ),
    tag = "Workflow"
)]
pub async fn create(
    State(ctx): State<Arc<Context>>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse> {
    Ok((StatusCode::CREATED, Json(WorkflowService::create(ctx, &req).await?)))
}

/// Delete a workflow
#[utoipa::path(
    operation_id = "delete-workflow",
    delete, path = "/v1/workflows/{id}",
    params(
        ("id" = Uuid, description = "The id of workflow"),
    ),
    responses(
        (status = 204, description = "Workflow deleted successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to delete workflow")
    ),
    tag = "Workflow"
)]
pub async fn delete(
    State(ctx): State<Arc<Context>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    WorkflowService::delete(ctx, id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get a workflow
#[utoipa::path(
    operation_id = "get-workflow-detail",
    get, path = "/v1/workflows/{id}",
    params(
        ("id" = Uuid, description = "The id of workflow"),
    ),
    responses(
        (status = 200, description = "Workflow retrieved successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to get workflow")
    ),
    tag = "Workflow"
)]
pub async fn get(
    State(ctx): State<Arc<Context>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    Ok((StatusCode::OK, Json(WorkflowService::get(ctx, id).await?)))
}
