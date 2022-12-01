use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::Addr;
use std::env::current_dir;
use std::fs::create_dir_all;

use minter::msg::{AddrBal, AddressValMsg, ConfigResponse, TokenDataResponse};
use minter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use minter::state::{CollectionInfo, Config};

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
    export_schema_with_title(
        &schema_for!(AddressValMsg),
        &out_dir,
        "CheckAddressMintsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressValMsg>),
        &out_dir,
        "GetAddressMintsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddrBal>),
        &out_dir,
        "GetEscrowBalancesResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<(u64, CollectionInfo)>),
        &out_dir,
        "GetCw721CollectionInfoResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<(Addr, u32)>),
        &out_dir,
        "GetBundleMintTrackerResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<(u64, u32)>),
        &out_dir,
        "GetCollectionCurrentTokenSupplyResponse",
    );
    export_schema_with_title(
        &schema_for!(TokenDataResponse),
        &out_dir,
        "GetRemainingTokensResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressValMsg>),
        &out_dir,
        "GetCW721AddrsResponse",
    );
}
