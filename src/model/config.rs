use std::error::Error;
#[derive(SqlObject)]
pub struct Config{
    pub id: Option<i64>,
    pub name: String,
    pub value: String
}

impl Config{
    pub fn new(name: &'static str, value: &str) -> Self{
        Config{
            id: None,
            name: String::from(name),
            value: String::from(value)
        }
    }
}