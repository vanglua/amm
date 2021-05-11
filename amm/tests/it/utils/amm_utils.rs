use crate::utils::*;

pub struct AMMUtils {
    pub contract: ContractAccount<AMMContract>
}

impl AMMUtils {
    pub fn new(master_account: &UserAccount, gov_id: AccountId) -> Self {
        // deploy amm
        let contract = deploy!(
            // Contract Proxy
            contract: AMMContractContract,
            // Contract account id
            contract_id: AMM_CONTRACT_ID,
            // Bytes of contract
            bytes: &AMM_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account,
            deposit: to_yocto("1000"),
            // init method
            init_method: init(
                gov_id.try_into().unwrap(),
                vec![amm::collateral_whitelist::Token{account_id: "token".to_string(), decimals: 24}]
            )
        );

        storage_deposit(TOKEN_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(AMM_CONTRACT_ID.to_string()));
        storage_deposit(ORACLE_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(AMM_CONTRACT_ID.to_string()));

        Self {
            contract
        }
    }
}