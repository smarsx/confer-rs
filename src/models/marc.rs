use tokio_postgres::{Client, Row};
use anyhow::Error;

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

    pub async fn get_open(client: &Client) -> Result<Vec<Marc>, Error> {
        let rows: Vec<Row> = client.query("SELECT name, address, status FROM Marc WHERE status = 1", &[]).await?;
        
        let res: Vec<Marc> = rows.iter()
            .map(|m| Marc::from_row(m))
            .collect();

        Ok(res)
    }
}
