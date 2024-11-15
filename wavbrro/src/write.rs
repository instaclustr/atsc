/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::path::Path;

pub fn write_wavbrro_file(file_path: &Path, content: &[u8]) {
    let header: [u8; 12] = *b"WBRO0000WBRO";
    let file = File::create(file_path).expect("Can't create file!");
    file.write_at(&header, 0).expect("Fail to write header");
    file.write_at(content, header.len() as u64)
        .expect("Fail to write content");
}
