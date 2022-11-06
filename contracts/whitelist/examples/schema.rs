use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

use whitelist::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use whitelist::state::Config;
use whitelist::msg::{CheckWhitelistResponse, ConfigResponse };

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);

    export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "GetConfigResponse");
    export_schema_with_title(&schema_for!(CheckWhitelistResponse), &out_dir, "CheckWhitelistResponse");
    export_schema_with_title(&schema_for!(Vec<String>), &out_dir, "GetWhitelistAddressesResponse");
    export_schema_with_title(&schema_for!(Vec<(String, u32)>), &out_dir, "GetAddressMintsResponse");
    export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "GetConfigResponse");
}
