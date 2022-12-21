use std::env;

static VALID_CHAINS: &'static [i32] = &[1, 10, 137, 42161];

pub fn get_env(env: &str) -> String {
    let var = &env::var(env);
    match var {
        Ok(v) => return String::from(v),
        Err(_) => panic!("invalid env var: {}", env)
    }
}

pub fn get_conn_str() -> String {
    get_env("PG_CONNSTR")
}

pub fn get_infura_key() -> String {
    get_env("INFURA_KEY")
}

pub fn get_optimism_key() -> String {
    get_env("ALCHEMY_OPTIMISM_KEY")
}

pub fn get_polygon_key() -> String {
    get_env("ALCHEMY_POLYGON_KEY")
}

pub fn get_chain_id() -> i32 {
    let chain_id: i32 = get_env("CHAIN_ID")
        .parse()
        .unwrap();

    if VALID_CHAINS.contains(&chain_id) {} else {
        panic!("invalid ChainId: {}, accepted values: {:?}", chain_id, VALID_CHAINS);
    }
    chain_id
}
