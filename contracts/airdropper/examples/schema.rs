use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use airdropper::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use airdropper::state::Config;

use airdropper::msg::{
    AddressPromisedTokensResponse, AddressTokenMsg, AddressValMsg,
    CheckAirdropPromisedMintResponse, CheckAirdropPromisedTokensResponse, TokenMsg,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);

    export_schema_with_title(&schema_for!(Config), &out_dir, "GetConfigResponse");
    export_schema_with_title(
        &schema_for!(Vec<AddressPromisedTokensResponse>),
        &out_dir,
        "GetAddressPromisedTokenIDsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<TokenMsg>),
        &out_dir,
        "GetAssignedTokenIDsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressTokenMsg>),
        &out_dir,
        "GetAssignedTokenIDsWithAddressResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<u32>),
        &out_dir,
        "GetClaimedTokenIDsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressValMsg>),
        &out_dir,
        "GetClaimedTokenIDsWithAddressResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressValMsg>),
        &out_dir,
        "GetAddressPromisedMintsResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<AddressValMsg>),
        &out_dir,
        "GetClaimedAddressPromisedMintsResponse",
    );
    export_schema_with_title(
        &schema_for!(CheckAirdropPromisedMintResponse),
        &out_dir,
        "CheckAddressPromisedMintsResponse",
    );
    export_schema_with_title(
        &schema_for!(CheckAirdropPromisedTokensResponse),
        &out_dir,
        "CheckAddressPromisedTokensResponse",
    );
}
