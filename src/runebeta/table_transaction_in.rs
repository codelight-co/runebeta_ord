use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::runebeta::models::NewTransactionIn;
use crate::schema::transaction_ins::dsl::*;
use crate::InsertRecords;
pub const NUMBER_OF_FIELDS: u16 = 7;
#[derive(Clone)]
pub struct TransactionInTable {}

impl<'conn> TransactionInTable {
  pub fn new() -> Self {
    Self {}
  }
  // pub fn inserts(
  //   &self,
  //   txs: &[NewTransactionIn],
  //   connection: &mut PgConnection,
  // ) -> Result<usize, diesel::result::Error> {
  //   diesel::insert_into(transaction_ins::table())
  //     .values(txs)
  //     .on_conflict_do_nothing()
  //     .execute(connection)
  // }
}

impl InsertRecords for TransactionInTable {
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionIn;
  fn insert_slice(
    &self,
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_ins::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
