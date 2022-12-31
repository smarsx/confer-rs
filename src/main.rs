use std::str::FromStr;
use dotenv::dotenv;
use ethers::{
    providers::{Provider, Http, Middleware, StreamExt, JsonRpcClient}, 
    utils::to_checksum, 
    types::{H160, H256, Filter, BlockNumber, TxHash, U64, Log}
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

    // backfill missing transactions (transactions from last recorded -> current)
    backfill_hashes(provider.clone(), marcs.clone()).await?;
    
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

async fn backfill_hashes<P: JsonRpcClient + 'static>(provider: Provider<P>, filter_addresses: Vec<H160>) -> anyhow::Result<()> {
    let connstr = get_conn_str();
    println!("backfiller connecting to pg");
    let mut pg = PgDB::connect(&connstr).await?;

    let last_hash = pg.get_last_transaction_hash().await?;
    let mut bn: U64 = U64::from_str("0").unwrap();
    
    // if last hash not 0 get block of last hash and overwrite bn
    if last_hash == "0" {} else {
        match TxHash::from_str(last_hash.as_str()) {
            Ok(h) => {
                let receipt = provider.get_transaction_receipt(h).await?;
                if let Some(r) = receipt {
                    if let Some(b) = r.block_number {
                        bn = b;
                    }
                }
            },
            Err(e) => {panic!("invalid hash from pg. {}", e)}
        }
    }

    let cb: U64 = provider.get_block_number().await?;
    let cbi: u32 = cb.as_u32();
    let mut bni: u32 = bn.as_u32();
    let interval: u32 = 25000;
    let mut count = 0;
    
    println!("backfilling hashes from {} to {}", bni, cbi);
    while bni < cbi {
        println!("bn: {}", bni);
        // create filter in block interval
        let filter = Filter::new()
            .from_block(bni)
            .to_block(bni + interval)
            .address(filter_addresses.clone());

        // get logs in filter
        let mut logs: Vec<Log> = provider.get_logs(&filter).await?;
        logs.sort_by(|l1, l2| l1.block_number.cmp(&l2.block_number));
        
        // get hashes from logs
        let hashes: Vec<H256> = logs.iter()
            .map(|l| l.transaction_hash.unwrap())
            .collect();
        
        // add hashes to db
        for hash in hashes {
            // bottleneck
            let res = pg.insert_transaction(&hash).await?;
            if res == false {
                panic!("bad resp from pg.insert_transaction")
            } else {
                count = count + 1;
            }
        }

        bni = bni + interval;
    }
    println!("added {} hashes", count);
    Ok(())
}
