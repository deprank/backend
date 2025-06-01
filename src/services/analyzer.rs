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

use std::path::{Path, PathBuf};

use anyhow::Result;

#[allow(dead_code)]
pub struct AnalyzerService {
    cache_dir: PathBuf,
}

impl AnalyzerService {
    pub fn new(cache_dir: &Path) -> Self {
        Self { cache_dir: cache_dir.to_path_buf() }
    }

    pub async fn analyze(&self, _dir: &Path) -> Result<()> {
        todo!()
    }
}
