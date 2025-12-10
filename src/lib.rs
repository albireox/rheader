/*
 *  @Author: José Sánchez-Gallego (gallegoj@uw.edu)
 *  @Date: 2025-12-09
 *  @Filename: lib.rs
 *  @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)
 */

pub mod header;
mod python;
pub mod tools;

// Re-exporting modules
pub use crate::header::{FITSValue, Header, Keyword, read_header};
pub use crate::tools::is_gzip_file;
