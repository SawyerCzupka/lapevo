use crate::error::{IracingError, Result};
use crate::vars::{VarType, VariableSchema};

/// A raw telemetry frame: byte buffer + schema reference for typed access.
#[derive(Debug, Clone)]
pub struct RawFrame {
    data: Vec<u8>,
}

impl RawFrame {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_f32(&self, schema: &VariableSchema, name: &str) -> Result<f32> {
        let var = schema.require(name)?;
        if var.var_type != VarType::Float32 {
            return Err(IracingError::Parse {
                context: name.to_string(),
                details: format!("expected Float32, got {:?}", var.var_type),
            });
        }
        self.read_f32(var.offset)
    }

    pub fn get_f64(&self, schema: &VariableSchema, name: &str) -> Result<f64> {
        let var = schema.require(name)?;
        if var.var_type != VarType::Float64 {
            return Err(IracingError::Parse {
                context: name.to_string(),
                details: format!("expected Float64, got {:?}", var.var_type),
            });
        }
        self.read_f64(var.offset)
    }

    pub fn get_i32(&self, schema: &VariableSchema, name: &str) -> Result<i32> {
        let var = schema.require(name)?;
        if !matches!(var.var_type, VarType::Int32 | VarType::BitField) {
            return Err(IracingError::Parse {
                context: name.to_string(),
                details: format!("expected Int32/BitField, got {:?}", var.var_type),
            });
        }
        self.read_i32(var.offset)
    }

    pub fn get_bool(&self, schema: &VariableSchema, name: &str) -> Result<bool> {
        let var = schema.require(name)?;
        if var.var_type != VarType::Bool {
            return Err(IracingError::Parse {
                context: name.to_string(),
                details: format!("expected Bool, got {:?}", var.var_type),
            });
        }
        self.data
            .get(var.offset)
            .map(|&b| b != 0)
            .ok_or_else(|| IracingError::Parse {
                context: name.to_string(),
                details: "offset out of bounds".to_string(),
            })
    }

    fn read_f32(&self, offset: usize) -> Result<f32> {
        let bytes = self.slice(offset, 4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_f64(&self, offset: usize) -> Result<f64> {
        let bytes = self.slice(offset, 8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_i32(&self, offset: usize) -> Result<i32> {
        let bytes = self.slice(offset, 4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn slice(&self, offset: usize, len: usize) -> Result<&[u8]> {
        self.data.get(offset..offset + len).ok_or_else(|| IracingError::Parse {
            context: "frame read".to_string(),
            details: format!("offset {} + {} exceeds frame size {}", offset, len, self.data.len()),
        })
    }
}
