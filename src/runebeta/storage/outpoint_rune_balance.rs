use bitcoin::OutPoint;
use diesel::{
  associations::HasTable, query_dsl::methods::FilterDsl, ExpressionMethods, PgConnection,
  RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{NewOutpointRuneBalance, OutpointRuneBalance},
  schema::outpoint_rune_balances::dsl::*,
};

pub struct OutpointRuneBalaneTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> OutpointRuneBalaneTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(
    &mut self,
    payload: &NewOutpointRuneBalance,
  ) -> Result<OutpointRuneBalance, diesel::result::Error> {
    diesel::insert_into(outpoint_rune_balances::table())
      .values(payload)
      .returning(OutpointRuneBalance::as_returning())
      .get_result(self.connection)
  }
  pub fn inserts(
    &mut self,
    payload: &Vec<NewOutpointRuneBalance>,
  ) -> Result<OutpointRuneBalance, diesel::result::Error> {
    diesel::insert_into(outpoint_rune_balances::table())
      .values(payload)
      .get_result(self.connection)
  }

  // pub fn get(&self, outpoint: &OutPoint) -> Result<Option<i64>, diesel::result::Error> {
  //   outpoint_values
  //     .filter(tx_hash.eq(outpoint.txid.to_string()))
  //     .filter(vout.eq(outpoint.vout))
  //     .returning(OutPointValue::as_returning())
  //     .get_result(self.connection)
  //     .map(|record| record.value)
  //   //.expect("Error saving satpoint")
  // }
}
