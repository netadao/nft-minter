use cosmwasm_schema::cw_serde;

use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};

use crate::msg::{Admin, ExecuteMsg, ModuleInstantiateInfo};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[cw_serde]
pub struct CwTemplateContract(pub Addr, pub u64, pub u64, pub u64);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn cw721_contract_id(&self) -> u64 {
        self.1
    }

    pub fn airdrop_contract_id(&self) -> u64 {
        self.2
    }

    pub fn whitelist_contract_id(&self) -> u64 {
        self.3
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

impl ModuleInstantiateInfo {
    pub fn into_wasm_msg(self, contract_addr: Addr) -> WasmMsg {
        WasmMsg::Instantiate {
            admin: match self.admin {
                Admin::Address { address } => Some(address),
                Admin::CoreContract {} => Some(contract_addr.to_string()),
                Admin::None {} => None,
            },
            code_id: self.code_id,
            msg: self.msg,
            funds: vec![],
            label: self.label,
        }
    }
}
