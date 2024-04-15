use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use super::models::NewOutpointRuneBalance;
use crate::{schema::outpoint_rune_balances::dsl::*, InsertRecords};
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct OutpointRuneBalanceTable {}

impl<'conn> OutpointRuneBalanceTable {
  pub fn new() -> Self {
    Self {}
  }
  // pub fn insert(
  //   &self,
  //   balances: &[NewOutpointRuneBalance],
  //   connection: &mut PgConnection,
  // ) -> Result<usize, diesel::result::Error> {
  //   diesel::insert_into(outpoint_rune_balances::table())
  //     .values(balances)
  //     .on_conflict_do_nothing()
  //     //.returning(OutpointRuneBalance::as_returning())
  //     .execute(connection)
  // }
}

impl InsertRecords for OutpointRuneBalanceTable {
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewOutpointRuneBalance;
  fn insert_slice(
    &self,
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(outpoint_rune_balances::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
