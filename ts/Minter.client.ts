/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { CheckAddressMintsResponse, Addr, Uint128, Timestamp, Uint64, CheckedDenom, Config, SharedCollectionInfo, RoyaltyInfo, ExecuteMsg, UncheckedDenom, Admin, Binary, ExecutionTarget, CosmosMsgForEmpty, BankMsg, StakingMsg, DistributionMsg, IbcMsg, WasmMsg, GovMsg, VoteOption, BaseInitMsg, ModuleInstantiateInfo, Coin, Empty, IbcTimeout, IbcTimeoutBlock, GetAddressMintsResponse, AddressValMsg, GetBundleMintTrackerResponse, GetCW721AddrsResponse, GetCollectionCurrentTokenSupplyResponse, GetConfigResponse, GetCw721CollectionInfoResponse, CollectionInfo, GetEscrowBalancesResponse, AddrBal, GetRemainingTokensResponse, InstantiateMsg, CollectionInfoMsg, SharedCollectionInfoMsg, RoyaltyInfoMsg, QueryMsg } from "./Minter.types";
export interface MinterReadOnlyInterface {
  contractAddress: string;
  getConfig: () => Promise<GetConfigResponse>;
  checkAddressMints: ({
    minterAddress
  }: {
    minterAddress: string;
  }) => Promise<CheckAddressMintsResponse>;
  getAddressMints: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<GetAddressMintsResponse>;
  getEscrowBalances: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<GetEscrowBalancesResponse>;
  getCw721CollectionInfo: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }) => Promise<GetCw721CollectionInfoResponse>;
  getBundleMintTracker: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<GetBundleMintTrackerResponse>;
  getCollectionCurrentTokenSupply: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }) => Promise<GetCollectionCurrentTokenSupplyResponse>;
  getRemainingTokens: () => Promise<GetRemainingTokensResponse>;
  getCW721Addrs: () => Promise<GetCW721AddrsResponse>;
}
export class MinterQueryClient implements MinterReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getConfig = this.getConfig.bind(this);
    this.checkAddressMints = this.checkAddressMints.bind(this);
    this.getAddressMints = this.getAddressMints.bind(this);
    this.getEscrowBalances = this.getEscrowBalances.bind(this);
    this.getCw721CollectionInfo = this.getCw721CollectionInfo.bind(this);
    this.getBundleMintTracker = this.getBundleMintTracker.bind(this);
    this.getCollectionCurrentTokenSupply = this.getCollectionCurrentTokenSupply.bind(this);
    this.getRemainingTokens = this.getRemainingTokens.bind(this);
    this.getCW721Addrs = this.getCW721Addrs.bind(this);
  }

  getConfig = async (): Promise<GetConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_config: {}
    });
  };
  checkAddressMints = async ({
    minterAddress
  }: {
    minterAddress: string;
  }): Promise<CheckAddressMintsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      check_address_mints: {
        minter_address: minterAddress
      }
    });
  };
  getAddressMints = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<GetAddressMintsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_address_mints: {
        limit,
        start_after: startAfter
      }
    });
  };
  getEscrowBalances = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<GetEscrowBalancesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_escrow_balances: {
        limit,
        start_after: startAfter
      }
    });
  };
  getCw721CollectionInfo = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }): Promise<GetCw721CollectionInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_cw721_collection_info: {
        limit,
        start_after: startAfter
      }
    });
  };
  getBundleMintTracker = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<GetBundleMintTrackerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_bundle_mint_tracker: {
        limit,
        start_after: startAfter
      }
    });
  };
  getCollectionCurrentTokenSupply = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }): Promise<GetCollectionCurrentTokenSupplyResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_collection_current_token_supply: {
        limit,
        start_after: startAfter
      }
    });
  };
  getRemainingTokens = async (): Promise<GetRemainingTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_remaining_tokens: {}
    });
  };
  getCW721Addrs = async (): Promise<GetCW721AddrsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_c_w721_addrs: {}
    });
  };
}
export interface MinterInterface extends MinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  updateConfig: ({
    bundleEnabled,
    bundleMintPrice,
    endTime,
    escrowFunds,
    maintainerAddress,
    maxPerAddressBundleMint,
    maxPerAddressMint,
    mintDenom,
    mintPrice,
    startTime
  }: {
    bundleEnabled: boolean;
    bundleMintPrice: Uint128;
    endTime?: Timestamp;
    escrowFunds: boolean;
    maintainerAddress?: string;
    maxPerAddressBundleMint: number;
    maxPerAddressMint: number;
    mintDenom: UncheckedDenom;
    mintPrice: Uint128;
    startTime: Timestamp;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  initSubmodule: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateWhitelistAddress: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateAirdropAddress: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  mint: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  mintBundle: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  airdropMint: ({
    minterAddress
  }: {
    minterAddress?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  airdropClaim: ({
    minterAddress
  }: {
    minterAddress?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  cleanClaimedTokensFromShuffle: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  shuffleTokenOrder: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  submoduleHook: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  disburseFunds: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class MinterClient extends MinterQueryClient implements MinterInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.updateConfig = this.updateConfig.bind(this);
    this.initSubmodule = this.initSubmodule.bind(this);
    this.updateWhitelistAddress = this.updateWhitelistAddress.bind(this);
    this.updateAirdropAddress = this.updateAirdropAddress.bind(this);
    this.mint = this.mint.bind(this);
    this.mintBundle = this.mintBundle.bind(this);
    this.airdropMint = this.airdropMint.bind(this);
    this.airdropClaim = this.airdropClaim.bind(this);
    this.cleanClaimedTokensFromShuffle = this.cleanClaimedTokensFromShuffle.bind(this);
    this.shuffleTokenOrder = this.shuffleTokenOrder.bind(this);
    this.submoduleHook = this.submoduleHook.bind(this);
    this.disburseFunds = this.disburseFunds.bind(this);
  }

  updateConfig = async ({
    bundleEnabled,
    bundleMintPrice,
    endTime,
    escrowFunds,
    maintainerAddress,
    maxPerAddressBundleMint,
    maxPerAddressMint,
    mintDenom,
    mintPrice,
    startTime
  }: {
    bundleEnabled: boolean;
    bundleMintPrice: Uint128;
    endTime?: Timestamp;
    escrowFunds: boolean;
    maintainerAddress?: string;
    maxPerAddressBundleMint: number;
    maxPerAddressMint: number;
    mintDenom: UncheckedDenom;
    mintPrice: Uint128;
    startTime: Timestamp;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_config: {
        bundle_enabled: bundleEnabled,
        bundle_mint_price: bundleMintPrice,
        end_time: endTime,
        escrow_funds: escrowFunds,
        maintainer_address: maintainerAddress,
        max_per_address_bundle_mint: maxPerAddressBundleMint,
        max_per_address_mint: maxPerAddressMint,
        mint_denom: mintDenom,
        mint_price: mintPrice,
        start_time: startTime
      }
    }, fee, memo, funds);
  };
  initSubmodule = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      init_submodule: {}
    }, fee, memo, funds);
  };
  updateWhitelistAddress = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_whitelist_address: {}
    }, fee, memo, funds);
  };
  updateAirdropAddress = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_airdrop_address: {}
    }, fee, memo, funds);
  };
  mint = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint: {}
    }, fee, memo, funds);
  };
  mintBundle = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_bundle: {}
    }, fee, memo, funds);
  };
  airdropMint = async ({
    minterAddress
  }: {
    minterAddress?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      airdrop_mint: {
        minter_address: minterAddress
      }
    }, fee, memo, funds);
  };
  airdropClaim = async ({
    minterAddress
  }: {
    minterAddress?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      airdrop_claim: {
        minter_address: minterAddress
      }
    }, fee, memo, funds);
  };
  cleanClaimedTokensFromShuffle = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      clean_claimed_tokens_from_shuffle: {}
    }, fee, memo, funds);
  };
  shuffleTokenOrder = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      shuffle_token_order: {}
    }, fee, memo, funds);
  };
  submoduleHook = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      submodule_hook: {}
    }, fee, memo, funds);
  };
  disburseFunds = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      disburse_funds: {}
    }, fee, memo, funds);
  };
}