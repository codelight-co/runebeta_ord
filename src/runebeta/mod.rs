pub mod extension;
mod models;
pub mod schema;
mod table_block;
mod table_outpoint_rune_balance;
mod table_transaction;
mod table_transaction_in;
mod table_transaction_out;
mod table_transaction_rune_entry;
use diesel::PgConnection;
pub use extension::IndexExtension;
pub use table_block::BlockTable;
pub use table_outpoint_rune_balance::OutpointRuneBalanceTable;
pub use table_transaction_in::TransactionInTable;
pub use table_transaction_out::TransactionOutTable;
pub use table_transaction_rune_entry::TransactionRuneEntryTable;
#[cfg(test)]
mod testing;

pub trait InsertRecords {
  const CHUNK_SIZE: usize;
  type Record;
  fn insert_vector(
    &self,
    records: &Vec<Self::Record>,
    conn: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    let len = records.len();
    let chunks = records.chunks(Self::CHUNK_SIZE);
    for chunk in chunks {
      let res = self.insert_slice(chunk, conn);
      if res.is_err() {
        log::info!("Insert error {:?}", &res);
        res?;
      }
    }
    Ok(len)
  }
  fn insert_slice(
    &self,
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error>;
}
