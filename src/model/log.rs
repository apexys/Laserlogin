#[derive(SqlMacro, Serialize, Clone, Debug)]
pub struct Log{
    pub id: i64,
    pub entry: String,
    pub timestamp_start: i64,
    pub timestamp_end: i64
}

impl Log{
    pub fn new(entry: String, timestamp_start: i64, timestamp_end: i64) -> Self{
        Log{
            id: -1,
            entry,
            timestamp_start,
            timestamp_end
        }
    }
}