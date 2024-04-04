use crate::{
  runebeta::{models::NewOutpointRuneBalance, OutpointRuneBalanceTable},
  Chain, RuneEntry, RuneId,
};

use super::{
  models::{
    CenotaphValue, NewBlock, NewTransaction, NewTransactionIn, NewTransactionOut, RunestoneValue,
  },
  table_transaction::TransactionTable,
  BlockTable, TransactionInTable, TransactionOutTable, TransactionRuneEntryTable,
};
use bigdecimal::BigDecimal;
use bitcoin::{
  block::Header, consensus::Encodable, opcodes, script::Instruction, Address, Transaction, TxIn,
  TxOut, Txid,
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use ordinals::{Artifact, Runestone};
use std::env;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct IndexExtension {
  chain: Chain,
  block_height: i64,
  block_header: Header,
  database_url: String,
}
impl IndexExtension {
  pub fn new(chain: Chain, block_height: i64, block_header: Header) -> Self {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let connection = PgConnection::establish(&database_url)
    //   .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    Self {
      chain,
      block_height,
      block_header,
      database_url,
    }
  }
  pub fn connect(&self) -> Result<PgConnection, ConnectionError> {
    PgConnection::establish(&self.database_url)
  }
  pub fn index_block(
    &self,
    block_data: &Vec<(Transaction, Txid)>,
  ) -> Result<usize, diesel::result::Error> {
    let new_block = NewBlock {
      block_time: self.block_header.time as i64,
      block_height: self.block_height,
      previous_hash: self.block_header.prev_blockhash.to_string(),
      block_hash: self.block_header.merkle_root.to_string(),
    };
    let mut vec_transactions = Vec::new();
    let mut vec_transaction_ins = Vec::new();
    let mut vec_transaction_outs = Vec::new();
    let mut vec_tx_ins: Vec<&TxIn> = Vec::new();
    for (tx, txid) in block_data.iter() {
      let artifact = Runestone::decipher(tx);
      let Transaction {
        version,
        lock_time,
        input,
        output,
      } = tx;
      let new_transaction = NewTransaction {
        version: *version,
        block_height: self.block_height,
        lock_time: lock_time.to_consensus_u32() as i32,
        tx_hash: txid.to_string(),
      };
      vec_transactions.push(new_transaction);
      input.iter().for_each(|tx_in| vec_tx_ins.push(tx_in));
      let mut new_transaction_ins = input
        .iter()
        .map(|txin| {
          let mut witness_buffer = Vec::new();
          let _ = txin.witness.consensus_encode(&mut witness_buffer);
          let mut witness = String::with_capacity(witness_buffer.len() * 2);
          for byte in witness_buffer.into_iter() {
            let _ = write!(&mut witness, "{:02X}", byte);
          }
          NewTransactionIn {
            tx_hash: txid.to_string(),
            previous_output_hash: txin.previous_output.txid.to_string(),
            previous_output_vout: txin.previous_output.vout as i32,
            script_sig: txin.script_sig.to_hex_string(),
            sequence_number: txin.sequence.0 as i64,
            witness,
          }
        })
        .collect();
      vec_transaction_ins.append(&mut new_transaction_ins);
      //Create transaction out for each transaction then push to common vector for whole block
      let mut new_transaction_outs = self.index_transaction_output(txid, output, artifact.as_ref());
      vec_transaction_outs.append(&mut new_transaction_outs);
    }
    let table_block = BlockTable::new();
    let table_tranction = TransactionTable::new();
    let table_transaction_in = TransactionInTable::new();
    let table_transaction_out = TransactionOutTable::new();
    let connection = self.connect();
    assert!(connection.is_ok());

    //must be safe to unwrap;
    let mut connection = connection.unwrap();
    let res = connection.build_transaction().read_write().run(|conn| {
      table_block.insert(&new_block, conn)?;
      table_tranction.inserts(&vec_transactions, conn)?;
      table_transaction_in.inserts(&vec_transaction_ins, conn)?;
      table_transaction_out.spends(&vec_tx_ins, conn)?;
      table_transaction_out.inserts(&vec_transaction_outs, conn)
    });
    if res.is_err() {
      log::debug!("Block index result {:?}", &res);
    }
    res
  }
  pub fn index_transaction_output(
    &self,
    txid: &Txid,
    output: &Vec<TxOut>,
    artifact: Option<&Artifact>,
  ) -> Vec<NewTransactionOut> {
    let new_transaction_outs = output
      .iter()
      .enumerate()
      .map(|(index, tx_out)| {
        let address = Address::from_script(&tx_out.script_pubkey, self.chain.network()).ok();
        let address = address.map(|addr| addr.to_string());

        let asm = tx_out.script_pubkey.to_asm_string();
        let dust_value = tx_out.script_pubkey.dust_value().to_sat() as i64;
        // Index op_return
        // payload starts with OP_RETURN
        // followed by the protocol identifier, ignoring errors, since OP_RETURN
        // scripts may be invalid
        let mut instructions = tx_out.script_pubkey.instructions();
        let mut runestone: Option<RunestoneValue> = None;
        let mut cenotaph: Option<CenotaphValue> = None;
        let mut edicts: i32 = 0;
        let mut burn = false;
        let mut etching = false;
        let mut mint = false;
        if instructions.next() == Some(Ok(Instruction::Op(opcodes::all::OP_RETURN)))
          && instructions.next() != Some(Ok(Instruction::Op(Runestone::MAGIC_NUMBER)))
          && artifact.is_some()
        {
          // construct the payload by concatenating remaining data pushes
          match artifact {
            Some(Artifact::Runestone(rs)) => {
              runestone = Some(RunestoneValue::from(rs));
              edicts = rs.edicts.len() as i32;
              etching = rs.etching.is_some();
              mint = rs.mint.is_some();
            }
            Some(Artifact::Cenotaph(v)) => {
              cenotaph = Some(CenotaphValue::from(v));
              etching = v.etching.is_some();
              mint = v.mint.is_some();
              burn = true;
            }
            None => {}
          };
        }

        NewTransactionOut {
          tx_hash: txid.to_string(),
          vout: index as i64,
          value: tx_out.value as i64,
          address,
          asm,
          dust_value,
          script_pubkey: tx_out.script_pubkey.to_hex_string(),
          spent: false,
          runestone: runestone.unwrap_or_default(),
          cenotaph: cenotaph.unwrap_or_default(),
          edicts,
          mint,
          etching,
          burn,
        }
      })
      .collect();
    new_transaction_outs
  }

  pub fn index_transaction_rune_entry(
    &self,
    txid: &Txid,
    rune_id: &RuneId,
    rune_entry: &RuneEntry,
  ) -> Result<usize, diesel::result::Error> {
    log::debug!("Runebeta index transaction rune {}, rune {}", txid, rune_id);
    let table_tranction_rune = TransactionRuneEntryTable::new();
    let connection = self.connect();
    assert!(connection.is_ok());
    //Must be safe to unwrap;
    let mut connection = connection.unwrap();
    let res = connection
      .build_transaction()
      .read_write()
      .run(|conn| table_tranction_rune.create(txid, rune_id, rune_entry, conn));
    log::debug!("Transaction rune index result {:?}", &res);
    res
  }

  pub fn index_outpoint_balances(
    &self,
    txid: &Txid,
    vout: i32,
    balances: &Vec<(RuneId, BigDecimal)>,
  ) -> Result<usize, diesel::result::Error> {
    log::debug!("Runebeta index outpoint balances of transaction {}", txid);
    let table_outpoint_balance = OutpointRuneBalanceTable::new();
    let outpoint_balances = balances
      .iter()
      .map(|(rune_id, balance)| NewOutpointRuneBalance {
        tx_hash: txid.to_string(),
        vout,
        rune_id: rune_id.to_string(),
        balance_value: balance.clone(),
      })
      .collect();
    let connection = self.connect();
    assert!(connection.is_ok());
    //Must be safe to unwrap;
    let mut connection = connection.unwrap();
    let res = connection
      .build_transaction()
      .read_write()
      .run(|conn| table_outpoint_balance.insert(&outpoint_balances, conn));
    if res.is_err() {
      log::debug!("Transaction rune index result {:?}", &res);
    }
    res
  }
}
