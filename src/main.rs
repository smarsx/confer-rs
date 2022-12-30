use std::str::FromStr;
use dotenv::dotenv;
use ethers::{
    providers::{Provider, Http, Middleware, StreamExt}, 
    utils::to_checksum, 
    types::{H160, H256, Filter, BlockNumber}
};

use confer_rs::{config::get_conn_str, utils::get_rpc_url, PgDB};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("loading .env");
    dotenv().ok();

    let connstr = get_conn_str();
    println!("connecting to postgre");
    let mut pg = PgDB::connect(&connstr).await?;

    let url = get_rpc_url();
    println!("connecting to rpc {}", &url);
    let provider = Provider::<Http>::try_from(url.as_str())?;

    let marcs: Vec<H160> = pg.get_marcs().await?
        .iter()
        .map(|m| m.address.to_owned())
        .map(|m| {
            let addr = H160::from_str(m.as_str());
            match addr {
                Ok(a) => return to_checksum(&a, Some(1)), // filter may auto checksum addresses?
                Err(_) => m.to_owned()
            }
        })
        .map(|m| H160::from_str(m.as_str()).unwrap())
        .collect();
    
    let filter = Filter::new().select(BlockNumber::Latest).address(marcs);

    // block iter
    let mut watcher = provider.watch_blocks().await?;

    while watcher.next().await.is_some() {
        let block_number = provider.get_block_number().await?;
        println!("\ngot block: {}", block_number.as_u64());

        // get hashes of filtered logs
        let hashes: Vec<H256> = provider.get_logs(&filter)
            .await?
            .iter()
            .map(|l| l.transaction_hash.unwrap())
            .collect();

        println!("found {} transactions", hashes.len());
        
        // add hashes to db
        for hash in hashes {
            let res = pg.insert_transaction(&hash).await?;
            if res == false {
                panic!("bad resp from pg.insert_transaction")
            }
        }

        // add block to db
        let block_insert = pg.insert_block(block_number.as_u64()).await?;
        if block_insert == false {
            panic!("bad resp from pg.insert_block")
        }
    }
    Ok(())
}
