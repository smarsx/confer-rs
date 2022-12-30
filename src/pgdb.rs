use ethers::types::H256;
use rust_decimal::prelude::*;
use tokio_postgres::{Client, NoTls};
use anyhow::{Result, Error};
use crate::models::Marc;

pub struct PgDB {
    client: Client
}

impl PgDB {
    pub async fn connect(connstr: &str) -> anyhow::Result<Self> {
        let (client, connection) = tokio_postgres::connect(connstr, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Self {
            client
        })
    }

    pub async fn get_marcs(&mut self) -> Result<Vec<Marc>, Error> {
        let rows = self.client.query("SELECT name, address, status FROM Marc WHERE status = 1", &[]).await?;
        let res: Vec<Marc> = rows.iter()
            .map(|m| Marc::from_row(m))
            .collect();
        
        Ok(res)
    }

    pub async fn insert_transaction(&mut self, hash: &H256) -> Result<bool, Error> {
        let res = self.client.execute(
            format!(
                "INSERT INTO Confirmed(hash) VALUES ($1)"
            ).as_str(),
            &[&format!("{:?}", hash)]
        ).await?;

        let resp = Some(res) == Some(1 as u64);
        Ok(resp)
    }

    pub async fn insert_block(&mut self, block: u64) -> Result<bool, Error> {
        let res = self.client.execute(
            format!(
                "INSERT INTO Blocks(block) VALUES ($1)"
            ).as_str(),
            &[&Decimal::from(block)]
        ).await?;

        let resp = Some(res) == Some(1 as u64);
        Ok(resp)
    }
}
