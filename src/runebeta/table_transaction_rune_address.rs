use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::runebeta::models::NewTransactionIn;
use crate::schema::txid_rune_addresss::dsl::*;
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct TransactionRuneIdAddressTable {}

impl<'conn> TransactionRuneIdAddressTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn insert(
    &self,
    txs: &Vec<NewTransactionRuneAddress>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_rune_addresss::table())
      .values(txs)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
