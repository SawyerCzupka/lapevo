pub mod error;
pub mod ibt;
pub mod mapping;
pub mod playback;
pub mod raw_frame;
pub mod source;
pub mod vars;
mod yaml_utils;

#[cfg(all(feature = "live", target_os = "windows"))]
pub mod live;
#[cfg(all(feature = "live", target_os = "windows"))]
pub mod live_mmap;

#[cfg(all(feature = "pitwall", target_os = "windows"))]
pub mod pitwall_live;
#[cfg(all(feature = "pitwall", target_os = "windows"))]
pub use pitwall_live::PitwallLiveSource;

pub use error::IracingError;
pub use ibt::IbtFile;
pub use mapping::map_to_telemetry_frame;
pub use playback::IbtPlayback;
pub use raw_frame::RawFrame;
pub use source::IbtReplaySource;
pub use vars::{VarHeader, VarType, VariableSchema};
