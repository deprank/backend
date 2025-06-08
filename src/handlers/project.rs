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

//! The Project Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    context::Context, errors::Result, responses::project::ProjectResponse,
    services::project::ProjectService,
};

/// Get a project
#[utoipa::path(
    get, path = "/v1/projects/{owner}/{name}",
    params(
        ("owner" = String, description = "The owner of project"),
        ("name" = String, description = "The name of project")
    ),
    responses(
        (status = 200, description = "Project retrieved successfully", body = ProjectResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to get project")
    ),
    tag = "Projects"
)]
pub async fn get(
    State(ctx): State<Arc<Context>>,
    Path((owner, name)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    Ok((StatusCode::OK, Json(ProjectService::get(ctx, &owner, &name).await?)))
}
