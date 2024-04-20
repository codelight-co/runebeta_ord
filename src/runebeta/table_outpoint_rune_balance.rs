use std::{
  cmp,
  thread::{self, JoinHandle},
  time::Instant,
};

use bitcoin::Txid;
use diesel::{
  associations::HasTable,
  r2d2::{ConnectionManager, Pool},
  ExpressionMethods, PgConnection, RunQueryDsl,
};

use super::models::NewOutpointRuneBalance;
use crate::{
  schema::outpoint_rune_balances::dsl::*, split_input, InsertRecords, CONNECTION_POOL_SIZE,
};
pub const NUMBER_OF_FIELDS: u16 = 10;
#[derive(Clone)]
pub struct OutpointRuneBalanceTable {}

impl<'conn> OutpointRuneBalanceTable {
  pub fn new() -> Self {
    Self {}
  }
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
        //Loop until we success get connection from the pool
        loop {
          if let Ok(mut connection) = pool.get() {
            let start = Instant::now();
            let res = diesel::update(outpoint_rune_balances)
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
    }
    Ok(handles)
  }
}

impl InsertRecords for OutpointRuneBalanceTable {
  const TABLE_NAME: &'static str = "outpoint_rune_balance";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewOutpointRuneBalance;
  fn insert_slice(
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
