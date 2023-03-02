/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { CheckAddressMintsResponse, Addr, Uint128, Timestamp, Uint64, Config, SharedCollectionInfo, RoyaltyInfo, ExecuteMsg, ExecutionTarget, CosmosMsgForEmpty, BankMsg, StakingMsg, DistributionMsg, Binary, IbcMsg, WasmMsg, GovMsg, VoteOption, BaseInitMsg, Coin, Empty, IbcTimeout, IbcTimeoutBlock, TokenMsg, GetAddressMintsResponse, AddressValMsg, GetBundleMintTrackerResponse, GetCollectionCurrentTokenSupplyResponse, GetConfigResponse, GetCustomBundleMintTrackerResponse, GetCw721AddrsResponse, GetCw721CollectionInfoResponse, CollectionInfo, GetEscrowBalancesResponse, AddrBal, GetRemainingTokensResponse, Admin, InstantiateMsg, ModuleInstantiateInfo, CollectionInfoMsg, SharedCollectionInfoMsg, RoyaltyInfoMsg, QueryMsg } from "./Minter.types";
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
  getCustomBundleMintTracker: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }) => Promise<GetCustomBundleMintTrackerResponse>;
  getCollectionCurrentTokenSupply: ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: number;
  }) => Promise<GetCollectionCurrentTokenSupplyResponse>;
  getRemainingTokens: ({
    address
  }: {
    address?: string;
  }) => Promise<GetRemainingTokensResponse>;
  getCw721Addrs: () => Promise<GetCw721AddrsResponse>;
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
    this.getCustomBundleMintTracker = this.getCustomBundleMintTracker.bind(this);
    this.getCollectionCurrentTokenSupply = this.getCollectionCurrentTokenSupply.bind(this);
    this.getRemainingTokens = this.getRemainingTokens.bind(this);
    this.getCw721Addrs = this.getCw721Addrs.bind(this);
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
  getCustomBundleMintTracker = async ({
    limit,
    startAfter
  }: {
    limit?: number;
    startAfter?: string;
  }): Promise<GetCustomBundleMintTrackerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_custom_bundle_mint_tracker: {
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
  getRemainingTokens = async ({
    address
  }: {
    address?: string;
  }): Promise<GetRemainingTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_remaining_tokens: {
        address
      }
    });
  };
  getCw721Addrs = async (): Promise<GetCw721AddrsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_cw721_addrs: {}
    });
  };
}
export interface MinterInterface extends MinterReadOnlyInterface {
  contractAddress: string;
  sender: string;
  updateConfig: ({
    airdropperAddress,
    bundleEnabled,
    bundleMintPrice,
    endTime,
    escrowFunds,
    maintainerAddress,
    maxPerAddressBundleMint,
    maxPerAddressMint,
    mintDenom,
    mintPrice,
    startTime,
    whitelistAddress
  }: {
    airdropperAddress?: string;
    bundleEnabled: boolean;
    bundleMintPrice: Uint128;
    endTime?: Timestamp;
    escrowFunds: boolean;
    maintainerAddress?: string;
    maxPerAddressBundleMint: number;
    maxPerAddressMint: number;
    mintDenom: string;
    mintPrice: Uint128;
    startTime: Timestamp;
    whitelistAddress?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  mint: ({
    isPromisedMint,
    minterAddress
  }: {
    isPromisedMint: boolean;
    minterAddress?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  mintBundle: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  airdropClaim: ({
    limit,
    minterAddress
  }: {
    limit?: number;
    minterAddress?: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  cleanClaimedTokensFromShuffle: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  shuffleTokenOrder: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  submoduleHook: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  disburseFunds: ({
    address
  }: {
    address: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  processCustomBundle: ({
    contentCount,
    mintPrice,
    purge,
    tokens
  }: {
    contentCount: number;
    mintPrice: Uint128;
    purge: boolean;
    tokens?: TokenMsg[];
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  mintCustomBundle: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
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
    this.mint = this.mint.bind(this);
    this.mintBundle = this.mintBundle.bind(this);
    this.airdropClaim = this.airdropClaim.bind(this);
    this.cleanClaimedTokensFromShuffle = this.cleanClaimedTokensFromShuffle.bind(this);
    this.shuffleTokenOrder = this.shuffleTokenOrder.bind(this);
    this.submoduleHook = this.submoduleHook.bind(this);
    this.disburseFunds = this.disburseFunds.bind(this);
    this.processCustomBundle = this.processCustomBundle.bind(this);
    this.mintCustomBundle = this.mintCustomBundle.bind(this);
  }

  updateConfig = async ({
    airdropperAddress,
    bundleEnabled,
    bundleMintPrice,
    endTime,
    escrowFunds,
    maintainerAddress,
    maxPerAddressBundleMint,
    maxPerAddressMint,
    mintDenom,
    mintPrice,
    startTime,
    whitelistAddress
  }: {
    airdropperAddress?: string;
    bundleEnabled: boolean;
    bundleMintPrice: Uint128;
    endTime?: Timestamp;
    escrowFunds: boolean;
    maintainerAddress?: string;
    maxPerAddressBundleMint: number;
    maxPerAddressMint: number;
    mintDenom: string;
    mintPrice: Uint128;
    startTime: Timestamp;
    whitelistAddress?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_config: {
        airdropper_address: airdropperAddress,
        bundle_enabled: bundleEnabled,
        bundle_mint_price: bundleMintPrice,
        end_time: endTime,
        escrow_funds: escrowFunds,
        maintainer_address: maintainerAddress,
        max_per_address_bundle_mint: maxPerAddressBundleMint,
        max_per_address_mint: maxPerAddressMint,
        mint_denom: mintDenom,
        mint_price: mintPrice,
        start_time: startTime,
        whitelist_address: whitelistAddress
      }
    }, fee, memo, funds);
  };
  mint = async ({
    isPromisedMint,
    minterAddress
  }: {
    isPromisedMint: boolean;
    minterAddress?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint: {
        is_promised_mint: isPromisedMint,
        minter_address: minterAddress
      }
    }, fee, memo, funds);
  };
  mintBundle = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_bundle: {}
    }, fee, memo, funds);
  };
  airdropClaim = async ({
    limit,
    minterAddress
  }: {
    limit?: number;
    minterAddress?: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      airdrop_claim: {
        limit,
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
  disburseFunds = async ({
    address
  }: {
    address: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      disburse_funds: {
        address
      }
    }, fee, memo, funds);
  };
  processCustomBundle = async ({
    contentCount,
    mintPrice,
    purge,
    tokens
  }: {
    contentCount: number;
    mintPrice: Uint128;
    purge: boolean;
    tokens?: TokenMsg[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      process_custom_bundle: {
        content_count: contentCount,
        mint_price: mintPrice,
        purge,
        tokens
      }
    }, fee, memo, funds);
  };
  mintCustomBundle = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      mint_custom_bundle: {}
    }, fee, memo, funds);
  };
}