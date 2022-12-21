use crate::config::{get_chain_id, get_infura_key, get_optimism_key, get_polygon_key};

pub fn get_rpc_url() -> String {
    match get_chain_id() {
        1 => {
            return String::from("http://localhost:8545")
        },
        5 => {
            let key = get_infura_key();
            return format!("https://goerli.infura.io/v3/{}", key)
        }
        10 => {
            let key = get_optimism_key();
            return format!("https://opt-mainnet.g.alchemy.com/v2/{}", key)
        }
        137 => {
            let key = get_polygon_key();
            return format!("https://polygon-mainnet.g.alchemy.com/v2/{}", key)
        }
        42161 => {
            return String::from("http://localhost:8547")
        }
        _ => {
            panic!("this should never raise")
        }
    }
}