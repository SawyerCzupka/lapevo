use std::collections::HashMap;

use crate::error::{IracingError, Result};

/// iRacing variable type enum matching irsdk_VarType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Char,    // 0
    Bool,    // 1
    Int32,   // 2
    BitField,// 3
    Float32, // 4
    Float64, // 5
}

impl VarType {
    pub fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(VarType::Char),
            1 => Some(VarType::Bool),
            2 => Some(VarType::Int32),
            3 => Some(VarType::BitField),
            4 => Some(VarType::Float32),
            5 => Some(VarType::Float64),
            _ => None,
        }
    }

    pub const fn size(self) -> usize {
        match self {
            VarType::Char | VarType::Bool => 1,
            VarType::Int32 | VarType::BitField | VarType::Float32 => 4,
            VarType::Float64 => 8,
        }
    }
}

/// Parsed variable header from IBT file.
#[derive(Debug, Clone)]
pub struct VarHeader {
    pub name: String,
    pub var_type: VarType,
    pub offset: usize,
    pub count: usize,
    pub units: String,
    pub description: String,
}

/// Schema describing all variables in a telemetry frame.
#[derive(Debug, Clone)]
pub struct VariableSchema {
    pub variables: HashMap<String, VarHeader>,
    pub frame_size: usize,
}

impl VariableSchema {
    pub fn get(&self, name: &str) -> Option<&VarHeader> {
        self.variables.get(name)
    }

    pub fn require(&self, name: &str) -> Result<&VarHeader> {
        self.get(name).ok_or_else(|| IracingError::MissingVariable(name.to_string()))
    }
}
