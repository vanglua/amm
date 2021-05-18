use crate::utils::*;

pub struct AMMUtils {
    pub contract: ContractAccount<AMMContract>
}

impl AMMUtils {
    pub fn new(master_account: &TestAccount, gov_id: AccountId) -> Self {
        // deploy amm
        let contract = deploy!(
            // Contract Proxy
            contract: AMMContractContract,
            // Contract account id
            contract_id: AMM_CONTRACT_ID,
            // Bytes of contract
            bytes: &AMM_WASM_BYTES,
            // User deploying the contract,
            signer_account: master_account.account,
            deposit: to_yocto("1000"),
            // init method
            init_method: init(
                gov_id.try_into().unwrap(),
                vec![amm::collateral_whitelist::Token{account_id: "token".to_string(), decimals: 24}],
                ORACLE_CONTRACT_ID.try_into().expect("invalid account id")
            )
        );

        storage_deposit(TOKEN_CONTRACT_ID, &master_account.account, SAFE_STORAGE_AMOUNT, Some(AMM_CONTRACT_ID.to_string()));
        storage_deposit(ORACLE_CONTRACT_ID, &master_account.account, SAFE_STORAGE_AMOUNT, Some(AMM_CONTRACT_ID.to_string()));

        Self {
            contract
        }
    }
}