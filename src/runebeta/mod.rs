pub mod extension;
mod models;
pub mod schema;
mod table_block;
mod table_outpoint_rune_balance;
mod table_transaction;
mod table_transaction_in;
mod table_transaction_out;
mod table_transaction_rune;
mod table_transaction_rune_address;
mod table_transaction_rune_entry;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
pub use extension::IndexExtension;
use std::thread::{self, JoinHandle};
use std::{fmt::Debug, time::Instant};
pub use table_block::BlockTable;
pub use table_outpoint_rune_balance::OutpointRuneBalanceTable;
pub use table_transaction_in::TransactionInTable;
pub use table_transaction_out::TransactionOutTable;
pub use table_transaction_rune::TransactionRuneTable;
pub use table_transaction_rune_address::TransactionRuneAddressTable;
pub use table_transaction_rune_entry::TransactionRuneEntryTable;
#[cfg(test)]
mod testing;
pub const CONNECTION_POOL_SIZE: usize = 10;
pub trait InsertRecords {
  const CHUNK_SIZE: usize;
  const TABLE_NAME: &'static str;
  type Record: 'static + Debug + Send;
  fn insert_vector(
    &self,
    records: Vec<Self::Record>,
    conn_pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Vec<JoinHandle<()>>, diesel::result::Error> {
    //let chunks = records.chunks(Self::CHUNK_SIZE);
    //1. Adjust chunk_size for split vector into chunk with equals length
    let chunk_size = calculate_chunk_size(records.len(), Self::CHUNK_SIZE);
    //2. Break input records into chunks
    let chunks = split_input(records, chunk_size);
    //3. In eacch iteration we get first Self::CHUNK_SIZE in remain vector then put into a single insert query
    let mut handles = vec![];
    for chunk in chunks {
      let pool = conn_pool.clone();

      let handle = thread::spawn(move || {
        //Move chunk into child thread
        let thread_chunk = chunk;
        //Loop until we success get connection from the pool
        loop {
          if let Ok(mut conn) = pool.get() {
            let start = Instant::now();
            let res = Self::insert_slice(&thread_chunk, &mut conn);
            if res.is_err() {
              log::info!("Insert error {:?}", res);
            } else {
              log::info!(
                "Inserted {} records in {} ms",
                thread_chunk.len(),
                start.elapsed().as_millis()
              );
            }
            break;
          }
        }
      });
      handles.push(handle);
    }
    Ok(handles)
  }
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error>;
  fn insert_record(
    &self,
    records: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error>;
}

//Calculate chunk size when device input vector
pub fn calculate_chunk_size(input_len: usize, max_size: usize) -> usize {
  if input_len <= max_size {
    input_len
  } else {
    let number_of_chunk = input_len / max_size;
    if number_of_chunk * max_size == input_len {
      max_size
    } else {
      //Total chunk number will be number_of_chunk + 1
      input_len / (number_of_chunk + 1)
    }
  }
}
/*
* Split the input vector into chunks for execute pg query
*/
pub fn split_input<T>(records: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
  let mut output_len = 0;
  let mut chunks = vec![Vec::<T>::with_capacity(chunk_size)];
  let mut cur_chunk = chunks.get_mut(output_len).expect("Chunks mut be not empty");
  output_len = output_len + 1;
  for item in records.into_iter() {
    // Create new chunk and push to final collection if it length reaches the chunk size
    if cur_chunk.len() == chunk_size {
      chunks.push(Vec::<T>::with_capacity(chunk_size));
      //Get reference to the latest chunk;
      cur_chunk = chunks.get_mut(output_len).expect("Chunks mut be not empty");
      output_len = output_len + 1;
    }
    cur_chunk.push(item);
  }
  chunks
}
