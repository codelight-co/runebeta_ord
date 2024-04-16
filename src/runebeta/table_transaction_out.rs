use bitcoin::{TxIn, Txid};
use diesel::{associations::HasTable, ExpressionMethods, PgConnection, RunQueryDsl};

use super::models::NewTransactionOut;
use crate::{schema::transaction_outs::dsl::*, InsertRecords};
pub const NUMBER_OF_FIELDS: u16 = 15;
#[derive(Clone)]
pub struct TransactionOutTable {}

impl<'conn> TransactionOutTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn inserts(
    &self,
    txs: &[NewTransactionOut],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(txs)
      .on_conflict_do_nothing()
      .execute(connection)
  }
  //Run in the same transaction as txin indexing
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
      diesel::update(transaction_outs)
        .filter(txout_id.eq_any(chunk))
        .set(spent.eq(true))
        .execute(connection)?;
    }
    Ok(txins.len())
  }

  pub fn spend(
    &self,
    txins: &Vec<TxIn>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    for txin in txins.iter() {
      diesel::update(transaction_outs)
        .filter(tx_hash.eq(txin.previous_output.txid.to_string().as_str()))
        .filter(vout.eq(txin.previous_output.vout as i64))
        .set(spent.eq(true))
        .execute(connection)?;
    }
    Ok(txins.len())
  }
}

impl InsertRecords for TransactionOutTable {
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionOut;
  fn insert_slice(
    &self,
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
