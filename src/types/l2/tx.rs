use crate::account::Signature;
use crate::types::fixnum::Float864;
use crate::types::merkle_tree::MerklePath;
use crate::types::primitives::{hash, u32_to_fr, Fr};
use anyhow::bail;
use anyhow::Result;
use ff::Field;
use std::convert::TryInto;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum TxType {
    DepositToNew,
    DepositToOld,
    Transfer,
    Withdraw,
    PlaceOrder,
    SpotTrade,
    Nop,
}

pub struct RawTx {
    pub tx_type: TxType,
    pub payload: Vec<Fr>,
    pub balance_path0: MerklePath,
    pub balance_path1: MerklePath,
    pub balance_path2: MerklePath,
    pub balance_path3: MerklePath,
    pub order_path0: MerklePath,
    pub order_path1: MerklePath,
    pub order_root0: Fr,
    pub order_root1: Fr,
    pub account_path0: MerklePath,
    pub account_path1: MerklePath,
    pub root_before: Fr,
    pub root_after: Fr,
    // debug info
    // extra: any;
}
pub type AmountType = Float864;

#[derive(Debug)]
pub struct PlaceOrderTx {
    pub order_id: u32,
    pub account_id: u32,
    pub token_id_sell: u32,
    pub token_id_buy: u32,
    pub amount_sell: Fr,
    pub amount_buy: Fr,
}

#[derive(Debug)]
pub struct DepositToOldTx {
    pub account_id: u32,
    pub token_id: u32,
    pub amount: AmountType,
}

#[derive(Debug)]
pub struct DepositToNewTx {
    pub account_id: u32,
    pub token_id: u32,
    pub amount: AmountType,
    pub eth_addr: Fr,
    pub sign: Fr,
    pub ay: Fr,
}

#[derive(Debug)]
pub struct SpotTradeTx {
    pub order1_account_id: u32,
    pub order2_account_id: u32,
    pub token_id_1to2: u32,
    pub token_id_2to1: u32,
    pub amount_1to2: AmountType,
    pub amount_2to1: AmountType,
    pub order1_id: u32,
    pub order2_id: u32,
}

#[derive(Debug)]
pub struct TransferTx {
    pub from: u32,
    pub to: u32,
    pub token_id: u32,
    pub amount: AmountType,
    pub from_nonce: Fr,
    pub to_nonce: Fr,
    pub old_balance_from: Fr,
    pub old_balance_to: Fr,
    pub sig: Signature,
}

impl TransferTx {
    pub fn new(from: u32, to: u32, token_id: u32, amount: AmountType) -> Self {
        Self {
            from,
            to,
            token_id,
            amount,
            from_nonce: Fr::zero(),
            to_nonce: Fr::zero(),
            old_balance_from: Fr::zero(),
            old_balance_to: Fr::zero(),
            sig: Signature::default(),
        }
    }

    pub fn hash(&self) -> Fr {
        let data = hash(&[u32_to_fr(TxType::Transfer as u32), u32_to_fr(self.token_id), self.amount.to_fr()]);
        // do we really need to sign oldBalance?
        let data = hash(&[data, u32_to_fr(self.from), self.from_nonce, self.old_balance_from]);
        hash(&[data, u32_to_fr(self.to), self.to_nonce, self.old_balance_to])
    }
}
pub const PUBDATA_LEN: usize = 60;
pub const ACCOUNT_ID_LEN: usize = 4;
pub const TOKEN_ID_LEN: usize = 2;
pub const AMOUNT_LEN: usize = 9;
//pub type PUBDATA = [u8; PUBDATA_LEN];

// https://github.com/Fluidex/circuits/issues/144
impl DepositToOldTx {
    pub fn to_pubdata(&self) -> Vec<u8> {
        let mut result = vec![TxType::DepositToOld as u8];
        result.append(&mut self.account_id.to_be_bytes().to_vec());
        result.append(&mut (self.token_id as u16).to_be_bytes().to_vec());
        result.append(&mut self.amount.encode());
        assert!(result.len() <= PUBDATA_LEN);
        result.append(&mut vec![0; PUBDATA_LEN - result.len()]);
        result
    }
    pub fn from_pubdata(data: &[u8]) -> Result<Self> {
        if data.len() != PUBDATA_LEN {
            bail!("invalid len for DepositToOldTx");
        }
        let mut idx: usize = 0;

        if data[0] != TxType::DepositToOld as u8 {
            bail!("invalid type for DepositToOldTx");
        }
        idx += 1;

        let account_id = u32::from_be_bytes(data[idx..(idx + ACCOUNT_ID_LEN)].try_into()?);
        idx += ACCOUNT_ID_LEN;

        let token_id = (u16::from_be_bytes(data[idx..(idx + TOKEN_ID_LEN)].try_into()?)) as u32;
        idx += TOKEN_ID_LEN;

        let amount = AmountType::decode(&data[idx..(idx + AMOUNT_LEN)])?;
        Ok(Self {
            account_id,
            token_id,
            amount,
        })
    }
}

#[cfg(test)]
#[test]
fn test_deposit_to_old_pubdata() {
    let tx = DepositToOldTx {
        account_id: 1323,
        token_id: 232,
        amount: AmountType {
            significand: 756,
            exponent: 11,
        },
    };
    let pubdata1 = tx.to_pubdata();
    println!("pubdata {:?}", pubdata1);
    let tx2 = DepositToOldTx::from_pubdata(&pubdata1).unwrap();
    assert_eq!(tx.account_id, tx2.account_id);
    assert_eq!(tx.token_id, tx2.token_id);
    assert_eq!(tx.amount.to_bigint(), tx2.amount.to_bigint());
}