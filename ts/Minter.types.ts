/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface CheckAddressMintsResponse {
  address: string;
  value: number;
}
export type Addr = string;
export type Uint128 = string;
export type Timestamp = Uint64;
export type Uint64 = string;
export interface Config {
  admin: Addr;
  bonded_denom: string;
  bundle_completed: boolean;
  bundle_enabled: boolean;
  bundle_mint_price: Uint128;
  custom_bundle_completed: boolean;
  custom_bundle_content_count: number;
  custom_bundle_enabled: boolean;
  custom_bundle_mint_price: Uint128;
  end_time?: Timestamp | null;
  escrow_funds: boolean;
  extension: SharedCollectionInfo;
  maintainer_addr?: Addr | null;
  max_per_address_bundle_mint: number;
  max_per_address_mint: number;
  mint_denom: string;
  mint_price: Uint128;
  start_time: Timestamp;
  token_code_id: number;
  total_token_supply: number;
}
export interface SharedCollectionInfo {
  mint_revenue_share: RoyaltyInfo[];
  secondary_market_royalties: RoyaltyInfo[];
}
export interface RoyaltyInfo {
  addr: Addr;
  bps: number;
  is_primary: boolean;
}
export type ExecuteMsg = {
  update_config: BaseInitMsg;
} | {
  mint: {
    is_promised_mint: boolean;
    minter_address?: string | null;
  };
} | {
  mint_bundle: {};
} | {
  airdrop_claim: {
    minter_address?: string | null;
  };
} | {
  clean_claimed_tokens_from_shuffle: {};
} | {
  shuffle_token_order: {};
} | {
  submodule_hook: [ExecutionTarget, CosmosMsgForEmpty];
} | {
  disburse_funds: {
    address: string;
  };
} | {
  process_custom_bundle: {
    content_count: number;
    mint_price: Uint128;
    purge: boolean;
    tokens?: TokenMsg[] | null;
  };
} | {
  mint_custom_bundle: {};
};
export type ExecutionTarget = "none" | "airdropper" | "whitelist";
export type CosmosMsgForEmpty = {
  bank: BankMsg;
} | {
  custom: Empty;
} | {
  staking: StakingMsg;
} | {
  distribution: DistributionMsg;
} | {
  stargate: {
    type_url: string;
    value: Binary;
    [k: string]: unknown;
  };
} | {
  ibc: IbcMsg;
} | {
  wasm: WasmMsg;
} | {
  gov: GovMsg;
};
export type BankMsg = {
  send: {
    amount: Coin[];
    to_address: string;
    [k: string]: unknown;
  };
} | {
  burn: {
    amount: Coin[];
    [k: string]: unknown;
  };
};
export type StakingMsg = {
  delegate: {
    amount: Coin;
    validator: string;
    [k: string]: unknown;
  };
} | {
  undelegate: {
    amount: Coin;
    validator: string;
    [k: string]: unknown;
  };
} | {
  redelegate: {
    amount: Coin;
    dst_validator: string;
    src_validator: string;
    [k: string]: unknown;
  };
};
export type DistributionMsg = {
  set_withdraw_address: {
    address: string;
    [k: string]: unknown;
  };
} | {
  withdraw_delegator_reward: {
    validator: string;
    [k: string]: unknown;
  };
};
export type Binary = string;
export type IbcMsg = {
  transfer: {
    amount: Coin;
    channel_id: string;
    timeout: IbcTimeout;
    to_address: string;
    [k: string]: unknown;
  };
} | {
  send_packet: {
    channel_id: string;
    data: Binary;
    timeout: IbcTimeout;
    [k: string]: unknown;
  };
} | {
  close_channel: {
    channel_id: string;
    [k: string]: unknown;
  };
};
export type WasmMsg = {
  execute: {
    contract_addr: string;
    funds: Coin[];
    msg: Binary;
    [k: string]: unknown;
  };
} | {
  instantiate: {
    admin?: string | null;
    code_id: number;
    funds: Coin[];
    label: string;
    msg: Binary;
    [k: string]: unknown;
  };
} | {
  migrate: {
    contract_addr: string;
    msg: Binary;
    new_code_id: number;
    [k: string]: unknown;
  };
} | {
  update_admin: {
    admin: string;
    contract_addr: string;
    [k: string]: unknown;
  };
} | {
  clear_admin: {
    contract_addr: string;
    [k: string]: unknown;
  };
};
export type GovMsg = {
  vote: {
    proposal_id: number;
    vote: VoteOption;
    [k: string]: unknown;
  };
};
export type VoteOption = "yes" | "no" | "abstain" | "no_with_veto";
export interface BaseInitMsg {
  airdropper_address?: string | null;
  bundle_enabled: boolean;
  bundle_mint_price: Uint128;
  end_time?: Timestamp | null;
  escrow_funds: boolean;
  maintainer_address?: string | null;
  max_per_address_bundle_mint: number;
  max_per_address_mint: number;
  mint_denom: string;
  mint_price: Uint128;
  start_time: Timestamp;
  whitelist_address?: string | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface Empty {
  [k: string]: unknown;
}
export interface IbcTimeout {
  block?: IbcTimeoutBlock | null;
  timestamp?: Timestamp | null;
  [k: string]: unknown;
}
export interface IbcTimeoutBlock {
  height: number;
  revision: number;
  [k: string]: unknown;
}
export interface TokenMsg {
  collection_id: number;
  token_id: number;
}
export type GetAddressMintsResponse = AddressValMsg[];
export interface AddressValMsg {
  address: string;
  value: number;
}
export type GetBundleMintTrackerResponse = [Addr, number][];
export type GetCollectionCurrentTokenSupplyResponse = [number, number][];
export interface GetConfigResponse {
  admin: Addr;
  airdropper_addr?: Addr | null;
  bundle_completed: boolean;
  bundle_enabled: boolean;
  bundle_mint_price: Uint128;
  custom_bundle_completed: boolean;
  custom_bundle_content_count: number;
  custom_bundle_enabled: boolean;
  custom_bundle_mint_price: Uint128;
  end_time?: Timestamp | null;
  escrow_funds: boolean;
  extension: SharedCollectionInfo;
  maintainer_addr?: Addr | null;
  max_per_address_bundle_mint: number;
  max_per_address_mint: number;
  mint_denom: string;
  mint_price: Uint128;
  start_time: Timestamp;
  token_code_id: number;
  total_token_supply: number;
  whitelist_addr?: Addr | null;
}
export type GetCustomBundleMintTrackerResponse = [Addr, number][];
export type GetCw721AddrsResponse = AddressValMsg[];
export type GetCw721CollectionInfoResponse = [number, CollectionInfo][];
export interface CollectionInfo {
  base_token_uri: string;
  id: number;
  name: string;
  symbol: string;
  token_supply: number;
}
export type GetEscrowBalancesResponse = AddrBal[];
export interface AddrBal {
  addr: Addr;
  balance: Uint128;
}
export interface GetRemainingTokensResponse {
  address_bundles_minted: number;
  address_minted: number;
  max_per_address_bundle_mint: number;
  max_per_address_mint: number;
  remaining_bundle_mints: number;
  remaining_custom_bundle_mints: number;
  remaining_token_supply: number;
  total_token_supply: number;
}
export type Admin = {
  address: {
    address: string;
  };
} | {
  core_contract: {};
} | {
  none: {};
};
export interface InstantiateMsg {
  airdropper_instantiate_info?: ModuleInstantiateInfo | null;
  base_fields: BaseInitMsg;
  collection_infos: CollectionInfoMsg[];
  extension: SharedCollectionInfoMsg;
  name: string;
  token_code_id: number;
  whitelist_instantiate_info?: ModuleInstantiateInfo | null;
}
export interface ModuleInstantiateInfo {
  admin: Admin;
  code_id: number;
  label: string;
  msg: Binary;
}
export interface CollectionInfoMsg {
  base_token_uri: string;
  name: string;
  symbol: string;
  token_supply: number;
}
export interface SharedCollectionInfoMsg {
  mint_revenue_share: RoyaltyInfoMsg[];
  secondary_market_royalties: RoyaltyInfoMsg[];
}
export interface RoyaltyInfoMsg {
  address: string;
  bps: number;
  is_primary: boolean;
}
export type QueryMsg = {
  get_config: {};
} | {
  check_address_mints: {
    minter_address: string;
  };
} | {
  get_address_mints: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  get_escrow_balances: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  get_cw721_collection_info: {
    limit?: number | null;
    start_after?: number | null;
  };
} | {
  get_bundle_mint_tracker: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  get_custom_bundle_mint_tracker: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  get_collection_current_token_supply: {
    limit?: number | null;
    start_after?: number | null;
  };
} | {
  get_remaining_tokens: {
    address?: string | null;
  };
} | {
  get_cw721_addrs: {};
};