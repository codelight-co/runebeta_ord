use std::{
  cmp,
  thread::{self, JoinHandle},
  time::Instant,
};

use super::models::NewTransactionOut;
use crate::{schema::transaction_outs::dsl::*, split_input, InsertRecords, CONNECTION_POOL_SIZE};
use bitcoin::Txid;
use diesel::{
  associations::HasTable,
  r2d2::{ConnectionManager, Pool},
  ExpressionMethods, PgConnection, RunQueryDsl,
};
pub const NUMBER_OF_FIELDS: u16 = 18;
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
    conn_pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Vec<JoinHandle<()>>, diesel::result::Error> {
    let mut handles = vec![];
    let txout_ids = txins
      .iter()
      .map(|(txid, ind)| format!("{}:{}", txid.to_string(), ind))
      .collect::<Vec<String>>();
    //Split update into small query for improve performance
    let chunk_size = cmp::min(u16::MAX as usize, txout_ids.len() / CONNECTION_POOL_SIZE);
    let chunks = split_input(txout_ids, chunk_size);

    for chunk in chunks {
      let pool = conn_pool.clone();

      let handle = thread::spawn(move || {
        //Move chunk into child thread
        let thread_chunk = chunk;
        loop {
          if let Ok(mut connection) = pool.get() {
            let start = Instant::now();
            let res = diesel::update(transaction_outs)
              .filter(txout_id.eq_any(&thread_chunk))
              .set(spent.eq(true))
              .execute(&mut connection);
            match res {
              Ok(size) => {
                log::info!(
                  "Updated {} records into the table {} in {} ms",
                  size,
                  Self::TABLE_NAME,
                  start.elapsed().as_millis()
                );
              }
              Err(err) => {
                log::info!("Updated error {:?}", &err);
              }
            }
            break;
          }
        }
      });
      handles.push(handle);
      // diesel::update(transaction_outs)
      //   .filter(txout_id.eq_any(chunk))
      //   .set(spent.eq(true))
      //   .execute(connection)?;
    }
    Ok(handles)
  }
}

impl InsertRecords for TransactionOutTable {
  const TABLE_NAME: &'static str = "transaction_outs";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionOut;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
