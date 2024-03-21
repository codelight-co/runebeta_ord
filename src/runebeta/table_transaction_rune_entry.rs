use bitcoin::Txid;
use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::transaction_rune_entries::dsl::*;
use crate::{RuneEntry, RuneId};

use super::models::{MintEntryType, NewTxRuneEntry, U128};
#[derive(Clone)]
pub struct TransactionRuneEntryTable {}

impl<'conn> TransactionRuneEntryTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn create(
    &self,
    txid: &Txid,
    rune_id: &RuneId,
    rune_entry: &RuneEntry,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    let etching_value = rune_entry.etching.to_string();
    let symbol_value = rune_entry.symbol.map(|c| c.to_string());
    let tx_rune_entry = NewTxRuneEntry {
      tx_hash: txid.to_string(),
      rune_height: rune_id.block as i32,
      rune_index: rune_id.tx as i16,
      burned: U128(rune_entry.burned),
      divisibility: rune_entry.divisibility as i16,
      etching: etching_value.as_str(),
      mints: rune_entry.mints as i64,
      number: rune_entry.number as i64,
      rune: U128(rune_entry.rune.0),
      spacers: rune_entry.spacers as i32,
      supply: U128(rune_entry.supply),
      symbol: symbol_value.as_ref().map(|c| c.as_str()),
      timestamp: rune_entry.timestamp as i32,
      mint_entry: rune_entry
        .mint
        .map(|entry| MintEntryType::from(&entry))
        .unwrap_or_default(),
    };
    diesel::insert_into(transaction_rune_entries::table())
      .values(tx_rune_entry)
      .execute(connection)
  }
}
