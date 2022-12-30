use tokio_postgres::Row;

pub struct Marc {
    pub name: String,
    pub address: String,
    pub status: i32,
}

impl Marc {
    pub fn from_row(row: &Row) -> Self {
        Self {
            name: row.get("name"),
            address: row.get("address"),
            status: row.get("status"),
        }
    }
}
