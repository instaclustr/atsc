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
use std::io::{self, Write};
use std::path::Path;

/// Header written in front of every WAVBRRO body: magic, format marker, magic.
pub const FILE_HEADER: [u8; 12] = *b"WBRO0001WBRO";

/// Writes `FILE_HEADER` followed by an archived WAVBRRO body.
pub fn try_write_wavbrro_file(file_path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(&FILE_HEADER)?;
    file.write_all(content)
}

pub fn write_wavbrro_file(file_path: &Path, content: &[u8]) {
    try_write_wavbrro_file(file_path, content).expect("Fail to write WAVBRRO file");
}
