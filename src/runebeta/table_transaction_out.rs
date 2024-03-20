use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use super::models::NewTransactionOut;
use crate::schema::transaction_outs::dsl::*;

#[derive(Clone)]
pub struct TransactionOutTable {}

impl<'conn> TransactionOutTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn insert(
    &self,
    txs: &Vec<NewTransactionOut>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(txs)
      .execute(connection)
  }
}
