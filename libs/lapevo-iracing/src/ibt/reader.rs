use std::path::{Path, PathBuf};

use crate::error::{IracingError, Result};
use crate::ibt::header::{
    IRSDK_VAR_HEADER_SIZE, IbtDiskSubHeader, IbtHeader, extract_variable_schema,
};
use crate::ibt::session::parse_session_info;
use crate::mapping::map_to_telemetry_frame;
use crate::raw_frame::RawFrame;
use crate::vars::VariableSchema;
use crate::yaml_utils;
use lapevo_telemetry::{SessionInfo, TelemetryFrame};
use tracing::warn;

/// IBT file reader implementing `TelemetryReader`.
pub struct IbtFile {
    data: Vec<u8>,
    header: IbtHeader,
    #[allow(dead_code)]
    disk_header: IbtDiskSubHeader,
    schema: VariableSchema,
    session: SessionInfo,
    frame_data_start: usize,
    total_frames: usize,
}

impl IbtFile {
    /// Open an IBT file for reading.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path.as_ref()).map_err(IracingError::Io)?;
        Self::from_bytes(data, path.as_ref().to_path_buf())
    }

    fn from_bytes(data: Vec<u8>, _path: PathBuf) -> Result<Self> {
        let mut cursor = std::io::Cursor::new(&data);

        let header = IbtHeader::parse(&mut cursor)?;
        header.validate()?;
        let disk_header = IbtDiskSubHeader::parse(&mut cursor)?;
        let schema = extract_variable_schema(&mut cursor, &header)?;

        // Calculate frame data start
        let var_headers_end =
            header.var_header_offset as usize + header.num_vars as usize * IRSDK_VAR_HEADER_SIZE;

        let session_info_end = if header.session_info_len > 0 {
            (header.session_info_offset + header.session_info_len) as usize
        } else {
            var_headers_end
        };

        let frame_data_start = session_info_end.max(var_headers_end);

        let remaining = data.len().checked_sub(frame_data_start).ok_or_else(|| {
            IracingError::InvalidIbt("frame data start exceeds file size".to_string())
        })?;

        let total_frames = if header.buf_len > 0 {
            remaining / header.buf_len as usize
        } else {
            0
        };

        if disk_header.record_count > 0 && total_frames > 0 {
            let expected = disk_header.record_count as usize;
            if expected != total_frames {
                warn!(
                    "frame count mismatch: disk header={}, calculated={}",
                    expected, total_frames
                );
            }
        }

        // Parse session YAML
        let session = Self::parse_session(&data, &header, &schema)?;

        Ok(Self {
            data,
            header,
            disk_header,
            schema,
            session,
            frame_data_start,
            total_frames,
        })
    }

    fn parse_session(
        data: &[u8],
        header: &IbtHeader,
        _schema: &VariableSchema,
    ) -> Result<SessionInfo> {
        if header.session_info_len <= 0 || header.session_info_offset <= 0 {
            return Ok(SessionInfo {
                track_name: String::new(),
                track_id: 0,
                track_length: 0.0,
                car_name: String::new(),
                car_id: 0,
                session_type: String::new(),
                tick_rate: if header.tick_rate > 0 {
                    header.tick_rate as f64
                } else {
                    60.0
                },
            });
        }

        let raw_yaml = yaml_utils::extract_yaml_from_memory(
            data,
            header.session_info_offset,
            header.session_info_len,
        )?;

        if raw_yaml.trim().is_empty() {
            return Ok(SessionInfo {
                track_name: String::new(),
                track_id: 0,
                track_length: 0.0,
                car_name: String::new(),
                car_id: 0,
                session_type: String::new(),
                tick_rate: if header.tick_rate > 0 {
                    header.tick_rate as f64
                } else {
                    60.0
                },
            });
        }

        let cleaned = yaml_utils::preprocess_iracing_yaml(&raw_yaml)?;
        let tick_rate = if header.tick_rate > 0 {
            header.tick_rate as f64
        } else {
            60.0
        };
        parse_session_info(&cleaned, tick_rate)
    }

    /// Get the variable schema.
    pub fn schema(&self) -> &VariableSchema {
        &self.schema
    }

    /// Total number of frames in the file.
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Tick rate from header.
    pub fn tick_rate(&self) -> f64 {
        if self.header.tick_rate > 0 {
            self.header.tick_rate as f64
        } else {
            60.0
        }
    }

    /// Read a raw frame by index.
    pub fn raw_frame(&self, index: usize) -> Result<RawFrame> {
        if index >= self.total_frames {
            return Err(IracingError::Parse {
                context: "frame read".to_string(),
                details: format!("frame {} out of range (0..{})", index, self.total_frames),
            });
        }
        let frame_size = self.header.buf_len as usize;
        let start = self.frame_data_start + index * frame_size;
        let end = start + frame_size;
        Ok(RawFrame::new(self.data[start..end].to_vec()))
    }

    /// Read a TelemetryFrame by index.
    pub fn frame(&self, index: usize) -> Result<TelemetryFrame> {
        let raw = self.raw_frame(index)?;
        map_to_telemetry_frame(&raw, &self.schema)
    }
}

impl lapevo_telemetry::TelemetryReader for IbtFile {
    fn read_all(&self) -> lapevo_telemetry::error::Result<Vec<TelemetryFrame>> {
        let mut frames = Vec::with_capacity(self.total_frames);
        for i in 0..self.total_frames {
            frames.push(
                self.frame(i)
                    .map_err(lapevo_telemetry::TelemetryError::from)?,
            );
        }
        Ok(frames)
    }

    fn frames(
        &self,
    ) -> Box<dyn Iterator<Item = lapevo_telemetry::error::Result<TelemetryFrame>> + '_> {
        Box::new((0..self.total_frames).map(move |i| {
            self.frame(i)
                .map_err(lapevo_telemetry::TelemetryError::from)
        }))
    }

    fn frame_count(&self) -> usize {
        self.total_frames
    }

    fn session(&self) -> &SessionInfo {
        &self.session
    }
}
