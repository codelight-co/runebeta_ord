use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use super::models::{NewTransaction, Transaction};
use crate::schema::transactions::dsl::*;
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct TransactionTable {}

impl<'conn> TransactionTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn _insert(
    &self,
    tx: &NewTransaction,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(tx)
      .on_conflict(tx_hash)
      .do_update()
      .set(tx)
      .returning(Transaction::as_returning())
      .execute(connection)
  }
  pub fn inserts(
    &self,
    txs: &[NewTransaction],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(txs)
      .on_conflict_do_nothing()
      .returning(Transaction::as_returning())
      .execute(connection)
  }
}
