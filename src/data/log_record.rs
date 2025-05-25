#[derive(PartialEq, Eq)]
pub enum LogRecordType {
    //put normally
    Normal = 1,

    //tombstone
    Deleted = 2,
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
}

/// index info in memory
#[derive(Copy, Clone)]
pub struct LogRecordPos {
    pub(crate) file_id: u32,
    pub(crate) offset: u64,
}
