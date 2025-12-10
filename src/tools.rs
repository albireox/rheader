/*
 *  @Author: José Sánchez-Gallego (gallegoj@uw.edu)
 *  @Date: 2025-12-09
 *  @Filename: tools.rs
 *  @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)
 */

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Checks if a file is gzip-compressed by reading its magic number.
pub fn is_gzip_file<P: AsRef<Path>>(path: P) -> anyhow::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 2]; // Read the first two bytes

    file.read_exact(&mut buffer)?;

    Ok(buffer[0] == 0x1F && buffer[1] == 0x8B)
}
