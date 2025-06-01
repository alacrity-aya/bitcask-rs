use prost::length_delimiter_len;

pub(crate) const CRC_LENGTH: usize = 4;
pub(crate) const TYPE_LENGTH: usize = 1;

#[derive(PartialEq, Eq)]
pub enum LogRecordType {
    //put normally
    Normal = 1,

    //tombstone
    Deleted = 2,
}

impl LogRecordType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Normal,
            2 => Self::Deleted,
            _ => unreachable!("unknow log record type"),
        }
    }
}

pub struct LogRecord {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
    pub(crate) rec_type: LogRecordType,
}

impl LogRecord {
    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }

    pub fn get_crc(&mut self) -> u32 {
        todo!()
    }
}

/// index info in memory
#[derive(Copy, Clone)]
pub struct LogRecordPos {
    pub(crate) file_id: u32,
    pub(crate) offset: u64,
}

pub struct ReadLogRecord {
    pub(crate) record: LogRecord,
    pub(crate) size: usize,
}

pub fn max_log_record_header_size() -> usize {
    std::mem::size_of::<u8>() + length_delimiter_len(u32::MAX as usize) * 2
}
