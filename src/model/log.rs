use std::error::Error;
#[derive(SqlObject, Serialize, Clone, Debug)]
pub struct Log{
    pub id: Option<i64>,
    pub user_email: String,
    pub entry: String,
    pub timestamp_start: i64,
    pub timestamp_end: i64
}

impl Log{
    pub fn new(user_email: String, entry: String, timestamp_start: i64, timestamp_end: i64) -> Self{
        Log{
            id: None,
            user_email,
            entry,
            timestamp_start,
            timestamp_end
        }
    }
}