/*
 *  @Author: José Sánchez-Gallego (gallegoj@uw.edu)
 *  @Date: 2025-12-09
 *  @Filename: header.rs
 *  @License: BSD 3-clause (http://www.opensource.org/licenses/BSD-3-Clause)
 */

use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::LazyLock;

use bytes::Bytes;
use flate2::bufread::GzDecoder;
use regex::bytes::Regex;

/// A FITS header consisting of multiple keywords.
pub struct Header {
    /// The keywords in the header.
    pub keywords: Vec<Keyword>,
}

impl Header {
    /// Creates a new, empty `Header`.
    pub fn new() -> Self {
        Header {
            keywords: Vec::new(),
        }
    }

    /// Adds a keyword to the header.
    pub fn add_keyword(&mut self, keyword: Keyword) {
        self.keywords.push(keyword);
    }

    /// Retrieves a keyword by its name.
    pub fn get_keyword(&self, key: &str) -> Option<&Keyword> {
        self.keywords.iter().find(|k| k.name == key)
    }

    /// Returns the number of keywords in the header.
    pub fn num_keywords(&self) -> usize {
        self.keywords.len()
    }
}

impl IntoIterator for Header {
    type Item = (String, FITSValue, Option<String>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.keywords
            .into_iter()
            .map(|keyword| (keyword.name, keyword.value, keyword.comment))
            .collect::<Vec<Self::Item>>()
            .into_iter()
    }
}

/// A single FITS header keyword.
#[derive(Debug)]
pub struct Keyword {
    /// The name of the keyword.
    pub name: String,
    /// The value of the keyword.
    pub value: FITSValue,
    /// The comment associated with the keyword.
    pub comment: Option<String>,
    /// The raw byte representation of the keyword's value.
    raw_value: Bytes,
    /// Indicates whether the keyword was parsed successfully.
    valid: bool,
}

impl std::ops::Deref for Keyword {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.raw_value[..]
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(comment) = &self.comment {
            write!(f, "{} = {} / {}", self.name, self.value, comment)
        } else {
            write!(f, "{} = {}", self.name, self.value)
        }
    }
}

impl Keyword {
    /// Returns whether the keyword was parsed successfully.
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Represents a FITS keyword value.
#[derive(Debug)]
pub enum FITSValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    Invalid,
}

impl Display for FITSValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FITSValue::String(s) => write!(f, "{}", s),
            FITSValue::Integer(i) => write!(f, "{}", i),
            FITSValue::Float(fl) => write!(f, "{}", fl),
            FITSValue::Bool(b) => write!(f, "{}", if *b { "T" } else { "F" }),
            FITSValue::Null => write!(f, "NULL"),
            FITSValue::Invalid => write!(f, "INVALID"),
        }
    }
}

/// Parses a FITS keyword value from a byte slice.
pub fn parse_keyword_value(value: &[u8]) -> anyhow::Result<FITSValue> {
    let value_str = String::from_utf8_lossy(value).trim().to_string();

    let value = if value_str.starts_with('\'') && value_str.ends_with('\'') {
        let unquoted = value_str[1..value_str.len() - 1].trim_end().to_string();
        FITSValue::String(unquoted)
    } else if value_str.eq_ignore_ascii_case("T") {
        FITSValue::Bool(true)
    } else if value_str.eq_ignore_ascii_case("F") {
        FITSValue::Bool(false)
    } else if value_str.eq_ignore_ascii_case("NULL") {
        FITSValue::Null
    } else if let Ok(int_val) = value_str.parse::<i64>() {
        FITSValue::Integer(int_val)
    } else if let Ok(float_val) = value_str.parse::<f64>() {
        FITSValue::Float(float_val)
    } else if value_str.is_empty() {
        FITSValue::Null
    } else {
        return Err(anyhow::anyhow!("Unrecognized FITS value: {}", value_str));
    };

    Ok(value)
}

// Regular expression to parse FITS keywords. Defined here as a static because it's fairly
// expensive to compile and we want to reuse it.
static KEYWORD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([A-Z0-9_-]{1,8})\s*=\s*(?:('[^']*')|([^/\s]*))\s*(?:/\s*(.*))?").unwrap()
});

/// Reads a FITS header from the specified file path.
pub fn read_header<T: AsRef<Path>>(path: T) -> anyhow::Result<Header> {
    // Open the file in read-only mode with buffer.
    let reader = BufReader::new(File::open(&path)?);

    // Create a decoder that handles gzip files if necessary.
    let mut decoder: Box<dyn Read> = if crate::tools::is_gzip_file(&path).unwrap_or(false) {
        Box::new(GzDecoder::new(reader))
    } else {
        Box::new(reader)
    };

    let mut header_buf = Vec::new();
    let mut buf = [0u8; 2880];

    let end_re = Regex::new(r"(END)\s*$").unwrap();

    // Read the header in 2880-byte blocks until we find the END keyword.
    loop {
        decoder
            .read_exact(&mut buf)
            .expect("Failed to read exact number of bytes");

        if end_re.is_match(&buf) {
            let end_pos = end_re.find(&buf).unwrap().start();
            header_buf.extend_from_slice(&buf[..end_pos]);
            break;
        }

        header_buf.extend_from_slice(&buf);
    }

    // Create a new Header and parse keywords.
    let mut header = Header::new();

    for keyword_chunk in header_buf.chunks(80) {
        if let Some(caps) = KEYWORD_RE.captures(keyword_chunk) {
            let name = String::from_utf8_lossy(&caps[1]).trim().to_string();

            let raw_value = if let Some(val) = caps.get(2) {
                val.as_bytes().trim_ascii_end()
            } else if let Some(val) = caps.get(3) {
                val.as_bytes().trim_ascii_end()
            } else {
                &[]
            };

            let comment_string = if let Some(com) = caps.get(4) {
                String::from_utf8_lossy(com.as_bytes()).trim().to_string()
            } else {
                "".to_string()
            };

            // Handle empty comments.
            let comment = if comment_string.is_empty() {
                None
            } else {
                Some(comment_string)
            };

            // Convert the raw value to a FITSValue.
            let keyword = if let Ok(value) = parse_keyword_value(&raw_value) {
                Keyword {
                    name: name.clone(),
                    value,
                    comment,
                    raw_value: Bytes::copy_from_slice(raw_value),
                    valid: true,
                }
            } else {
                Keyword {
                    name: name.clone(),
                    value: FITSValue::Invalid,
                    comment,
                    raw_value: Bytes::copy_from_slice(raw_value),
                    valid: false,
                }
            };

            header.add_keyword(keyword);
        }
    }

    Ok(header)
}
