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
        handlers::project::get,

        handlers::dependency::list,
        handlers::dependency::get,

        handlers::contributor::list,
        handlers::contributor::get,

        handlers::workflow::get,
        handlers::workflow::create,
        handlers::workflow::delete,

        handlers::contribution::list,
        handlers::contribution::get,

        handlers::allocation::list,
        handlers::allocation::get,
    ),
    components(
        schemas(
            responses::project::ProjectResponse,

            requests::workflow::CreateWorkflowRequest,
            responses::workflow::WorkflowResponse,
        )
    ),
    tags(
        (name = "Projects", description = "The Project Service Handlers"),
        (name = "Workflows", description = "The Workflow Service Handlers"),
    ),
)]
struct ApiDoc;

pub fn build() -> SwaggerUi {
    SwaggerUi::new("/swagger").url("/openapi.json", ApiDoc::openapi())
}
