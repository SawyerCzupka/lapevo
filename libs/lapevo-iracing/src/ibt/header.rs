use crate::error::{IracingError, Result};
use crate::vars::{VarHeader, VarType, VariableSchema};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use tracing::debug;

const IRSDK_HEADER_SIZE: usize = 144;
const IRSDK_DISK_SUBHEADER_SIZE: usize = 32;
pub const IRSDK_VAR_HEADER_SIZE: usize = 144;
const IRSDK_VAR_NAME_SIZE: usize = 32;
const IRSDK_VAR_DESC_SIZE: usize = 64;
const IRSDK_VAR_UNIT_SIZE: usize = 32;

/// Parsed IBT file header (matches irsdk_header).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IbtHeader {
    pub version: i32,
    // #[allow(unused)]
    pub tick_rate: i32,
    pub session_info_update: i32,
    pub session_info_len: i32,
    pub session_info_offset: i32,
    pub num_vars: i32,
    pub var_header_offset: i32,
    pub buf_len: i32,
}

/// IBT disk sub-header with timing and record counts.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IbtDiskSubHeader {
    pub start_date: i64,
    pub start_time: f64,
    pub end_time: f64,
    pub lap_count: i32,
    pub record_count: i32,
}

impl IbtHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; IRSDK_HEADER_SIZE];
        reader
            .read_exact(&mut buf)
            .map_err(|e| IracingError::Parse {
                context: "IBT header".to_string(),
                details: format!("failed to read {} bytes: {}", IRSDK_HEADER_SIZE, e),
            })?;

        let version = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let tick_rate = i32::from_le_bytes(buf[8..12].try_into().unwrap());
        let session_info_update = i32::from_le_bytes(buf[12..16].try_into().unwrap());
        let session_info_len = i32::from_le_bytes(buf[16..20].try_into().unwrap());
        let session_info_offset = i32::from_le_bytes(buf[20..24].try_into().unwrap());
        let num_vars = i32::from_le_bytes(buf[24..28].try_into().unwrap());
        let var_header_offset = i32::from_le_bytes(buf[28..32].try_into().unwrap());
        let buf_len = i32::from_le_bytes(buf[36..40].try_into().unwrap());

        Ok(Self {
            version,
            tick_rate,
            session_info_update,
            session_info_len,
            session_info_offset,
            num_vars,
            var_header_offset,
            buf_len,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 2 {
            return Err(IracingError::Version {
                expected: 2,
                found: self.version,
            });
        }
        if self.num_vars < 0 || self.buf_len < 0 {
            return Err(IracingError::InvalidIbt(
                "negative header values".to_string(),
            ));
        }
        if self.num_vars > 10_000 || self.buf_len > 100_000_000 {
            return Err(IracingError::InvalidIbt(
                "unreasonable header values".to_string(),
            ));
        }
        Ok(())
    }
}

impl IbtDiskSubHeader {
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; IRSDK_DISK_SUBHEADER_SIZE];
        reader
            .read_exact(&mut buf)
            .map_err(|e| IracingError::Parse {
                context: "IBT disk sub-header".to_string(),
                details: format!("failed to read: {}", e),
            })?;

        Ok(Self {
            start_date: i64::from_le_bytes(buf[0..8].try_into().unwrap()),
            start_time: f64::from_le_bytes(buf[8..16].try_into().unwrap()),
            end_time: f64::from_le_bytes(buf[16..24].try_into().unwrap()),
            lap_count: i32::from_le_bytes(buf[24..28].try_into().unwrap()),
            record_count: i32::from_le_bytes(buf[28..32].try_into().unwrap()),
        })
    }
}

/// Extract variable schema from IBT file headers.
pub fn extract_variable_schema<R: Read + Seek>(
    reader: &mut R,
    header: &IbtHeader,
) -> Result<VariableSchema> {
    if header.buf_len == 0 || header.num_vars <= 0 {
        return Ok(VariableSchema {
            variables: HashMap::new(),
            frame_size: 0,
        });
    }

    reader
        .seek(SeekFrom::Start(header.var_header_offset as u64))
        .map_err(|e| IracingError::Parse {
            context: "var header seek".to_string(),
            details: e.to_string(),
        })?;

    let num_vars = header.num_vars as usize;
    let mut variables = HashMap::with_capacity(num_vars);

    for i in 0..num_vars {
        let mut buf = [0u8; IRSDK_VAR_HEADER_SIZE];
        reader
            .read_exact(&mut buf)
            .map_err(|e| IracingError::Parse {
                context: format!("var header {}", i),
                details: e.to_string(),
            })?;

        let var_type_raw = i32::from_le_bytes(buf[0..4].try_into().unwrap());
        let offset = i32::from_le_bytes(buf[4..8].try_into().unwrap());
        let count = i32::from_le_bytes(buf[8..12].try_into().unwrap());

        let name = extract_null_terminated(&buf[16..16 + IRSDK_VAR_NAME_SIZE]);
        let desc = extract_null_terminated(&buf[48..48 + IRSDK_VAR_DESC_SIZE]);
        let unit = extract_null_terminated(&buf[112..112 + IRSDK_VAR_UNIT_SIZE]);

        if name.is_empty() || offset < 0 || count <= 0 {
            continue;
        }

        let Some(var_type) = VarType::from_raw(var_type_raw) else {
            debug!(
                "skipping variable '{}' with unknown type {}",
                name, var_type_raw
            );
            continue;
        };

        variables.insert(
            name.clone(),
            VarHeader {
                name,
                var_type,
                offset: offset as usize,
                count: count as usize,
                units: unit,
                description: desc,
            },
        );
    }

    debug!(
        "extracted {} variables, frame_size={}",
        variables.len(),
        header.buf_len
    );
    Ok(VariableSchema {
        variables,
        frame_size: header.buf_len as usize,
    })
}

fn extract_null_terminated(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
