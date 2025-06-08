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

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{handlers, requests, responses};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::airdrop::get,
        handlers::airdrop::submit,

        handlers::allocation::get,
        handlers::allocation::list,

        handlers::contribution::get,
        handlers::contribution::list,

        handlers::contributor::get,
        handlers::contributor::list,

        handlers::dependency::get,
        handlers::dependency::list,

        handlers::project::get,

        handlers::wallet::bind,
        handlers::wallet::unbind,

        handlers::workflow::create,
        handlers::workflow::delete,
        handlers::workflow::get,
    ),
    components(
        schemas(
            requests::wallet::WalletAddressRequest,
            requests::workflow::CreateWorkflowRequest,

            responses::project::ProjectResponse,
            responses::workflow::WorkflowResponse,
        )
    ),
    tags(
        (name = "Airdrop", description = "The Airdrop Service Handlers"),
        (name = "Allocation", description = "The Allocation Service Handlers"),
        (name = "Contribution", description = "The Contribution Service Handlers"),
        (name = "Contributor", description = "The Contributor Service Handlers"),
        (name = "Dependency", description = "The Dependency Service Handlers"),
        (name = "Project", description = "The Project Service Handlers"),
        (name = "Wallet", description = "The Wallet address Service Handlers"),
        (name = "Workflow", description = "The Workflow Service Handlers"),
    ),
)]
struct ApiDoc;

pub fn build() -> SwaggerUi {
    SwaggerUi::new("/swagger").url("/openapi.json", ApiDoc::openapi())
}
