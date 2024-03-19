use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use super::{
  models::OutpointRuneBalance,
  storage::{
    ContentTypeCountTable, HomeInscriptionTable, InscriptionEntryTable, OutpointRuneBalaneTable,
    RuneEntryTable, RuneTable, SatPointTable, SequenceNumberRuneIdTable, TransactionTable,
    TxidRuneTable,
  },
};

pub fn establish_pgconnection() -> PgConnection {
  dotenv().ok();

  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  PgConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/*
 * Helper functions
 */

pub struct WriteTransaction {
  connection: PgConnection,
}

impl WriteTransaction {
  pub fn new() -> Self {
    let connection = establish_pgconnection();
    Self { connection }
  }
  pub fn create_rune_entry_table(&mut self) -> RuneEntryTable {
    RuneEntryTable::new(&mut self.connection)
  }
  pub fn create_rune_table(&mut self) -> RuneTable {
    RuneTable::new(&mut self.connection)
  }
  pub fn create_outpoint_rune_balance_table(&mut self) -> OutpointRuneBalaneTable {
    OutpointRuneBalaneTable::new(&mut self.connection)
  }
  pub fn create_sequence_number_runeid_table(&mut self) -> SequenceNumberRuneIdTable {
    SequenceNumberRuneIdTable::new(&mut self.connection)
  }
  pub fn create_txid_rune_table(&mut self) -> TxidRuneTable {
    TxidRuneTable::new(&mut self.connection)
  }
  pub fn create_transaction_table(&mut self) -> TransactionTable {
    TransactionTable::new(&mut self.connection)
  }
  pub fn create_inscription_entry_table(&mut self) -> InscriptionEntryTable {
    InscriptionEntryTable::new(&mut self.connection)
  }
  pub fn create_home_inscription_table(&mut self) -> HomeInscriptionTable {
    HomeInscriptionTable::new(&mut self.connection)
  }
  pub fn create_satpoint_table(&mut self) -> SatPointTable {
    SatPointTable::new(&mut self.connection)
  }
  pub fn create_content_type_count_table(&mut self) -> ContentTypeCountTable {
    ContentTypeCountTable::new(&mut self.connection)
  }
  // pub fn open_table<DieselTable>(&mut self, table: DieselTable) -> Result<Table, anyhow::Error> {
  //   let table = Table::new(&mut self.connection, table);
  //   Ok(table)
  // }
  pub fn get_connection(&self) -> &mut PgConnection {
    &mut self.connection
  }
}
