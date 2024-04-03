use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use super::models::NewOutpointRuneBalance;
use crate::schema::outpoint_rune_balances::dsl::*;
#[derive(Clone)]
pub struct OutpointRuneBalanceTable {}

impl<'conn> OutpointRuneBalanceTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn insert(
    &self,
    balances: &Vec<NewOutpointRuneBalance>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(outpoint_rune_balances::table())
      .values(balances)
      //.returning(OutpointRuneBalance::as_returning())
      .execute(connection)
  }
}
