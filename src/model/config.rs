#[derive(SqlMacro)]
pub struct Config{
    pub id: i64,
    pub name: String,
    pub value: String
}

impl Config{
    pub fn new(name: &'static str, value: &str) -> Self{
        Config{
            id: -1,
            name: String::from(name),
            value: String::from(value)
        }
    }
}