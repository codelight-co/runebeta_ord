use crate::{Rune, RuneEntry, RuneId};

use super::{
  models::{NewTransaction, NewTransactionIn, NewTransactionOut},
  table_transaction::TransactionTable,
  TransactionInTable, TransactionOutTable, TransactionRuneEntryTable,
};
use anyhow::Ok;
use bitcoin::{consensus::Encodable, Transaction, Txid};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::fmt::Write;

#[derive(Clone)]
pub struct IndexExtension {
  //connection: PgConnection,
  database_url: String,
}
impl IndexExtension {
  pub fn new() -> Self {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let connection = PgConnection::establish(&database_url)
    //   .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    Self { database_url }
  }
  pub fn connect(&self) -> Result<PgConnection, ConnectionError> {
    PgConnection::establish(&self.database_url)
  }
  pub fn index_transaction(&self, txid: &Txid, tx: &Transaction) -> Result<(), anyhow::Error> {
    let Transaction {
      version,
      lock_time,
      input,
      output,
    } = tx;
    let new_transaction = NewTransaction {
      version: *version,
      lock_time: lock_time.to_consensus_u32() as i32,
      tx_hash: txid.to_string(),
    };
    let new_transaction_outs = output
      .iter()
      .map(|tx_out| NewTransactionOut {
        tx_hash: txid.to_string(),
        value: tx_out.value as i64,
        script_pubkey: tx_out.script_pubkey.to_hex_string(),
      })
      .collect();
    let new_transaction_ins = input
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
          sequence_number: txin.sequence.0 as i32,
          witness,
        }
      })
      .collect();
    let mut connection = self.connect()?;
    let table_tranction = TransactionTable::new();
    let table_transaction_in = TransactionInTable::new();
    let table_transaction_out = TransactionOutTable::new();
    connection.build_transaction().read_write().run(|conn| {
      let _ = table_tranction.insert(&new_transaction, conn);
      let _ = table_transaction_in.insert(&new_transaction_ins, conn);
      let _ = table_transaction_out.insert(&new_transaction_outs, conn);
      Ok(())
    })?;

    Ok(())
  }
  pub fn index_transaction_rune(
    &self,
    txid: &Txid,
    rune_id: &RuneId,
    rune_entry: &RuneEntry,
  ) -> Result<(), anyhow::Error> {
    let table_tranction_rune = TransactionRuneEntryTable::new();
    let mut connection = self.connect()?;
    let _ = table_tranction_rune.create(txid, rune_id, rune_entry, &mut connection);
    Ok(())
  }
}
