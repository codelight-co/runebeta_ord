use bitcoin::Txid;
use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::transaction_rune_entries::dsl::*;
use crate::{RuneEntry, RuneId};

use super::models::{NewTxRuneEntry, TxRuneEntry};
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
    let tx_rune_entry = NewTxRuneEntry {
      tx_hash: todo!(),
      rune_height: todo!(),
      rune_index: todo!(),
      burned: todo!(),
      divisibility: todo!(),
      etching: todo!(),
      mints: todo!(),
      number: todo!(),
      rune: todo!(),
      spacers: todo!(),
      supply: todo!(),
      symbol: todo!(),
      timestamp: todo!(),
      mint_entry: todo!(),
    };
    diesel::insert_into(transaction_rune_entries::table())
      .values(&tx_rune_entry)
      .execute(connection)
  }
}
