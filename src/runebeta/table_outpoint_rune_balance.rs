use bitcoin::Txid;
use diesel::{associations::HasTable, ExpressionMethods, PgConnection, RunQueryDsl};

use super::models::NewOutpointRuneBalance;
use crate::{schema::outpoint_rune_balances::dsl::*, InsertRecords};
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct OutpointRuneBalanceTable {}

impl<'conn> OutpointRuneBalanceTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn spends(
    &self,
    txins: &Vec<(Txid, i64)>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    let txout_ids = txins
      .iter()
      .map(|(txid, ind)| format!("{}:{}", txid.to_string(), ind))
      .collect::<Vec<String>>();
    let chunks = txout_ids.chunks(u16::MAX as usize);
    for chunk in chunks {
      diesel::update(outpoint_rune_balances)
        .filter(txout_id.eq_any(chunk))
        .set(spent.eq(true))
        .execute(connection)?;
    }
    Ok(txins.len())
  }
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
  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(outpoint_rune_balances::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
