use near_sdk::serde::{Serialize, Deserialize};
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Token {
    pub account_id: AccountId,
    pub decimals: u32,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Whitelist(pub UnorderedMap<AccountId, u32>);

impl Whitelist {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut whitelist = UnorderedMap::new(b"wl".to_vec());

        for token in tokens.into_iter() {
            whitelist.insert(&token.account_id, &token.decimals);
        };
        Self(whitelist)
    }
}

#[near_bindgen]
impl AMMContract {
    /**
     * @returns the whitelisted collateral tokens
     */
    pub fn get_collateral_whitelist(&self) -> Vec<(AccountId, u32)> {
        self.collateral_whitelist.0.to_vec()
    }


    /**
     * @notice sets the list of tokens that are to be used as collateral
     * @param tokens list of `Token`s that can be used as collateral
     */
    pub fn set_collateral_whitelist(
        &mut self,
        tokens: Vec<Token>, 
    ) {
        self.assert_gov();
        self.collateral_whitelist = Whitelist::new(tokens);
    }

    /**
     * @notice add a single specified `AccountId` to the whitelist
     * @param to_add the `Token` to add
     */
    pub fn add_to_collateral_whitelist(
        &mut self,
        to_add: Token
    ) {
        self.assert_gov();
        self.collateral_whitelist.0.insert(&to_add.account_id, &to_add.decimals);
        logger::log_whitelist(&self.collateral_whitelist);
    }
}