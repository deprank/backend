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

//! The Wallet address Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{context::Context, errors::Result, requests::wallet::WalletAddressRequest};

/// Bind wallet address to workflow.
#[utoipa::path(
    operation_id = "bind-wallet-address",
    put, path = "/v1/workflows/{id}/wallet-address",
    params(
        ("id" = Uuid, description = "The id of workflow"),
    ),
    request_body(
        content = inline(WalletAddressRequest),
        description = "Bind wallet address request",
        content_type = "application/json"
    ),
    responses(
        (status = 204, description = "Wallet address bound successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to bind wallet address")
    ),
    tag = "Wallet"
)]
pub async fn bind(
    State(_ctx): State<Arc<Context>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<WalletAddressRequest>,
) -> Result<impl IntoResponse> {
    Ok(StatusCode::NO_CONTENT)
}

/// Unbind wallet address from workflow.
#[utoipa::path(
    operation_id = "unbind-wallet-address",
    delete, path = "/v1/workflows/{id}/wallet-address",
    params(
        ("id" = Uuid, description = "The id of workflow"),
    ),
    responses(
        (status = 204, description = "Wallet address unbound successfully"),
        (status = 404, description = "Workflow not found"),
        (status = 500, description = "Failed to unbind wallet address")
    ),
    tag = "Wallet"
)]
pub async fn unbind(
    State(_ctx): State<Arc<Context>>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    Ok(StatusCode::NO_CONTENT)
}
