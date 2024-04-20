use bigdecimal::BigDecimal;
use bitcoin::Txid;
use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::transaction_rune_entries::dsl::*;
use crate::{InsertRecords, RuneEntry, RuneId};

use super::models::{MintEntryType, NewTxRuneEntry};
pub const NUMBER_OF_FIELDS: u16 = 20;

#[derive(Clone)]
pub struct TransactionRuneEntryTable {}

impl<'conn> TransactionRuneEntryTable {
  pub fn new() -> Self {
    Self {}
  }
  // pub fn inserts(
  //   &self,
  //   entries: &[NewTxRuneEntry],
  //   connection: &mut PgConnection,
  // ) -> Result<usize, diesel::result::Error> {
  //   diesel::insert_into(transaction_rune_entries::table())
  //     .values(entries)
  //     .on_conflict_do_nothing()
  //     //.returning(OutpointRuneBalance::as_returning())
  //     .execute(connection)
  // }
  pub fn create(
    &self,
    txid: &Txid,
    rune_id_value: &RuneId,
    rune_entry: &RuneEntry,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    let etching_value = rune_entry.etching.to_string();
    let symbol_value = rune_entry.symbol.map(|c| c.to_string());
    let tx_rune_entry = NewTxRuneEntry {
      tx_hash: txid.to_string(),
      block_height: rune_id_value.block as i64,
      tx_index: rune_id_value.tx as i32,
      rune_id: rune_id_value.to_string(),
      burned: BigDecimal::from(rune_entry.burned),
      divisibility: rune_entry.divisibility as i16,
      etching: etching_value,
      parent: None,
      mints: rune_entry.mints as i64,
      number: rune_entry.block as i64,
      rune: BigDecimal::from(rune_entry.spaced_rune.rune.0),
      spacers: rune_entry.spaced_rune.spacers as i32,
      premine: rune_entry.premine as i64,
      spaced_rune: rune_entry.spaced_rune.to_string(),
      supply: BigDecimal::from(0_u128),
      symbol: symbol_value,
      timestamp: rune_entry.timestamp as i32,
      mint_entry: rune_entry
        .terms
        .map(|entry| MintEntryType::from(&entry))
        .unwrap_or_default(),
      turbo: rune_entry.turbo,
    };
    diesel::insert_into(transaction_rune_entries::table())
      .values(tx_rune_entry)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}

impl InsertRecords for TransactionRuneEntryTable {
  const TABLE_NAME: &'static str = "transaction_rune_entry";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTxRuneEntry;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_rune_entries::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_rune_entries::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
