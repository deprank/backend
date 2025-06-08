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

//! The Airdrop Service Handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{context::Context, errors::Result, requests::wallet::WalletAddressRequest};

/// Get airdrop detail.
#[utoipa::path(
    operation_id = "get-airdrop-detail",
    get, path = "/v1/airdrops/{id}",
    params(
        ("id" = Uuid, description = "The id of airdrop"),
    ),
    responses(
        (status = 200, description = "Airdrop retrieved successfully"),
        (status = 404, description = "Airdrop not found"),
        (status = 500, description = "Failed to get airdrop")
    ),
    tag = "Airdrop"
)]
pub async fn get(
    State(_ctx): State<Arc<Context>>,
    Path(_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    Ok(StatusCode::OK)
}

/// Submit wallet address to airdrop for receive.
#[utoipa::path(
    operation_id = "submit-airdop-wallet-address",
    post, path = "/v1/airdrops/{id}",
    params(
        ("id" = Uuid, description = "The id of airdrop"),
    ),
    request_body(
        content = inline(WalletAddressRequest),
        description = "Submit wallet address request",
        content_type = "application/json"
    ),
    responses(
        (status = 204, description = "Wallet address submitted successfully"),
        (status = 404, description = "Airdrop not found"),
        (status = 500, description = "Failed to get airdrop")
    ),
    tag = "Airdrop"
)]
pub async fn submit(
    State(_ctx): State<Arc<Context>>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<WalletAddressRequest>,
) -> Result<impl IntoResponse> {
    Ok(StatusCode::NO_CONTENT)
}
