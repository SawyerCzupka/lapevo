pub mod error;
pub mod ibt;
pub mod mapping;
pub mod playback;
pub mod raw_frame;
pub mod vars;
mod yaml_utils;

#[cfg(all(feature = "live", target_os = "windows"))]
pub mod live;
#[cfg(all(feature = "live", target_os = "windows"))]
pub mod live_mmap;

pub use error::IracingError;
pub use ibt::IbtFile;
pub use mapping::map_to_telemetry_frame;
pub use playback::IbtPlayback;
pub use raw_frame::RawFrame;
pub use vars::{VarHeader, VarType, VariableSchema};
