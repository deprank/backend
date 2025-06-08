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

//! The Dependency Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::{context::Context, errors::Result};

/// Get dependencies list of the project
#[utoipa::path(
    operation_id = "get-dependencies-list",
    get, path = "/v1/projects/{owner}/{name}/dependencies",
    params(
        ("owner" = String, description = "The owner of project"),
        ("name" = String, description = "The name of project"),
    ),
    responses(
        (status = 200, description = "Dependencies retrieved successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to get project")
    ),
    tag = "Projects"
)]
pub async fn list(
    State(_ctx): State<Arc<Context>>,
    Path((_owner, _name)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    Ok(Vec::new())
}

/// Get the dependency detail of the project
#[utoipa::path(
    operation_id = "get-dependency-detail",
    get, path = "/v1/projects/{owner}/{name}/dependencies/{dep}",
    params(
        ("owner" = String, description = "The owner of project"),
        ("name" = String, description = "The name of project"),
        ("dep" = String, description = "The name of dependency")
    ),
    responses(
        (status = 200, description = "Dependency retrieved successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Failed to get project")
    ),
    tag = "Projects"
)]
pub async fn get(
    State(_ctx): State<Arc<Context>>,
    Path((_owner, _name, _dep)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    Ok(Vec::new())
}
