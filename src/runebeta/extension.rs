use crate::{
  runebeta::{
    models::{MintEntryType, NewOutpointRuneBalance},
    OutpointRuneBalanceTable,
  },
  Chain, InsertRecords, RuneEntry, RuneId, TransactionRuneTable,
};

use super::{
  models::{
    CenotaphValue, NewBlock, NewTransaction, NewTransactionIn, NewTransactionOut,
    NewTransactionRune, NewTxRuneEntry, RunestoneValue,
  },
  table_transaction::TransactionTable,
  BlockTable, TransactionInTable, TransactionOutTable, TransactionRuneEntryTable,
};
use bigdecimal::BigDecimal;
use bitcoin::{
  block::Header, consensus::Encodable, opcodes, script::Instruction, Address, Transaction, TxOut,
  Txid,
};
use diesel::r2d2::Pool;
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use diesel_migrations::{
  embed_migrations, EmbeddedMigrations, HarnessWithOutput, MigrationHarness,
};
use dotenvy::dotenv;
use ordinals::{Artifact, Runestone};
use std::{
  collections::{HashMap, VecDeque},
  fmt::Write,
  sync::{Arc, Mutex, RwLock},
  thread::JoinHandle,
  time::{Instant, SystemTime, UNIX_EPOCH},
};
use std::{env, thread, time};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Default)]
pub struct IndexBlock {
  block_height: u64,
  // Raw txin for update previous txouts'spent => true
  raw_tx_ins: Mutex<Vec<(Txid, i64)>>,
  blocks: Mutex<Vec<NewBlock>>,
  transactions: Mutex<Vec<NewTransaction>>,
  transaction_ins: Mutex<Vec<NewTransactionIn>>,
  transaction_outs: Mutex<Vec<NewTransactionOut>>,
  //Store outpoint banlance in each rune update
  outpoint_balances: Mutex<Vec<NewOutpointRuneBalance>>,
  tx_runes: Mutex<Vec<NewTransactionRune>>,
  rune_entries: Mutex<Vec<NewTxRuneEntry>>,
}
impl IndexBlock {
  pub fn new(height: u64) -> Self {
    Self {
      block_height: height,
      ..Default::default()
    }
  }
  fn add_block(&self, block: NewBlock) {
    if let Ok(ref mut blocks) = self.blocks.try_lock() {
      blocks.push(block);
    } else {
      log::info!("Cannot lock the blocks for insert item");
    }
  }
  fn add_rune_entry(&self, rune_entry: NewTxRuneEntry) {
    if let Ok(ref mut rune_entries) = self.rune_entries.try_lock() {
      rune_entries.push(rune_entry);
    } else {
      log::info!("Cannot lock the rune_entries for insert item");
    }
  }
  fn append_transactions(&self, items: &mut Vec<NewTransaction>) {
    if let Ok(ref mut transactions) = self.transactions.try_lock() {
      transactions.append(items);
    } else {
      log::info!("Cannot lock the transactions for insert item");
    }
  }
  fn append_transaction_ins(&self, items: &mut Vec<NewTransactionIn>) {
    if let Ok(ref mut transaction_ins) = self.transaction_ins.try_lock() {
      transaction_ins.append(items);
    } else {
      log::info!("Cannot lock the transaction_ins for insert item");
    }
  }
  fn append_transaction_outs(&self, items: &mut Vec<NewTransactionOut>) {
    if let Ok(ref mut transaction_outs) = self.transaction_outs.try_lock() {
      transaction_outs.append(items);
    } else {
      log::info!("Cannot lock the transaction_outs for insert item");
    }
  }
  fn append_outpoint_rune_balances(&self, items: &mut Vec<NewOutpointRuneBalance>) {
    if let Ok(ref mut outpoint_balances) = self.outpoint_balances.try_lock() {
      outpoint_balances.append(items);
    } else {
      log::info!("Cannot lock the rune_balances for insert item");
    }
  }

  fn append_tx_rune(&self, items: &mut Vec<NewTransactionRune>) {
    if let Ok(ref mut tx_runes) = self.tx_runes.try_lock() {
      tx_runes.append(items);
    } else {
      log::info!("Cannot lock the tx_runes for insert item");
    }
  }
  fn append_raw_ins(&self, items: &mut Vec<(Txid, i64)>) {
    if let Ok(ref mut raw_tx_ins) = self.raw_tx_ins.try_lock() {
      raw_tx_ins.append(items);
    } else {
      log::info!("Cannot lock the raw_tx_ins for insert item");
    }
  }
  // pub fn get_size(&self) -> u128 {
  //   self.blocks.len() as u128
  //     + self.transactions.len() as u128
  //     + self.transaction_ins.len() as u128
  //     + self.transaction_outs.len() as u128
  //     + self.outpoint_balances.len() as u128
  //     + self.rune_entries.len() as u128
  //     + self.tx_ins.len() as u128
  // }
  // fn _clear(&mut self) {
  //   self.blocks.clear();
  //   self.transactions.clear();
  //   self.transaction_ins.clear();
  //   self.transaction_outs.clear();
  //   self.tx_ins.clear();
  //   self.rune_entries.clear();
  //   self.outpoint_balances.clear();
  // }
}
#[derive(Debug)]
pub struct IndexExtension {
  chain: Chain,
  //Apr 21
  //Todo: index "Rune transaction" only - Must deeply understand which txs are related with some runes
  _index_all_transaction: bool,
  last_block_height: u32,
  connection_pool: Pool<ConnectionManager<PgConnection>>,
  index_cache: RwLock<VecDeque<Arc<IndexBlock>>>,
  //Store indexer start time
  index_log: RwLock<HashMap<i64, u128>>,
}
impl IndexExtension {
  pub fn new(chain: Chain) -> Self {
    dotenv().ok();
    let index_all_transaction =
      env::var("ORD_SUPERSATS_INDEX_ALL_TRANSACTIONS").unwrap_or_else(|_| String::from("0"));
    let last_block_height = env::var("ORD_LAST_BLOCK_HEIGHT")
      .ok()
      .and_then(|val| val.parse::<u32>().ok())
      .unwrap_or(u32::MAX);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    let mut connection_pool = None;
    loop {
      let manager = ConnectionManager::<PgConnection>::new(database_url.as_str());
      let Ok(pool) = Pool::builder().test_on_check_out(true).build(manager) else {
        let ten_second = time::Duration::from_secs(10);
        thread::sleep(ten_second);
        log::info!("Try connect to reconnect to the db");
        continue;
      };
      //Run db migration
      if let Ok(mut connection) = pool.clone().get() {
        let mut harness_with_output = HarnessWithOutput::write_to_stdout(&mut connection);
        let res = harness_with_output.run_pending_migrations(MIGRATIONS);
        if res.is_err() {
          log::info!("Run migration with error {:?}", &res);
        }
      };
      connection_pool.replace(pool);
      break;
    }

    Self {
      chain,
      last_block_height,
      _index_all_transaction: index_all_transaction == "1",
      connection_pool: connection_pool.expect("Connection pool must successfull created"),
      index_cache: Default::default(),
      index_log: Default::default(),
    }
  }
  pub fn get_latest_block_height(&self) -> u32 {
    self.last_block_height
  }

  pub fn get_block_cache(&self, height: u64) -> Option<Arc<IndexBlock>> {
    if let Ok(cache) = self.index_cache.read() {
      for cache in cache.iter() {
        if cache.block_height == height {
          //get existing cache
          return Some(Arc::clone(cache));
        }
      }
    }
    None
  }
  pub fn add_index_block(&self, index_block: Arc<IndexBlock>) {
    if let Ok(mut cache) = self.index_cache.write() {
      cache.push_back(index_block);
    }
  }
  pub fn log_start_indexing(&self, height: i64) {
    let current = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards");
    log::info!(
      "Indexer start index block {} at {}",
      &height,
      current.as_millis()
    );
    let mut index_log = self.index_log.write().unwrap();
    index_log.insert(height.clone(), current.as_millis());
  }
  pub fn log_finish_indexing(&self, height: i64) {
    let current = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis();
    let start = self
      .index_log
      .read()
      .unwrap()
      .get(&height)
      .map(|v| v.clone())
      .unwrap_or_default();

    log::info!(
      "[Benchmark]#{}|{}|{}|{}",
      &height,
      start,
      current,
      current - start
    );
  }
  /*
   * This function is call after index other info
   */
  pub fn index_block(
    &self,
    height: i64,
    block_header: &Header,
    block_data: &Vec<(Transaction, Txid)>,
  ) -> Result<(), diesel::result::Error> {
    self.log_start_indexing(height);
    let new_block = NewBlock {
      block_time: block_header.time as i64,
      block_height: height,
      previous_hash: block_header.prev_blockhash.to_string(),
      block_hash: block_header.block_hash().to_string(),
    };
    let mut transactions = vec![];
    let mut transaction_outs = vec![];
    let mut transaction_ins: Vec<NewTransactionIn> = vec![];
    let mut tx_ins = vec![];

    for (tx_index, (tx, txid)) in block_data.iter().enumerate() {
      let tx_hash = txid.to_string();
      let artifact = Runestone::decipher(tx);
      let Transaction {
        version,
        lock_time,
        input,
        output,
      } = tx;
      let new_transaction = NewTransaction {
        version: *version,
        block_height: height,
        tx_index: tx_index as i32,
        lock_time: lock_time.to_consensus_u32() as i64,
        tx_hash,
      };
      transactions.push(new_transaction);
      input.iter().for_each(|tx_in| {
        tx_ins.push((
          tx_in.previous_output.txid.clone(),
          tx_in.previous_output.vout as i64,
        ))
      });
      let mut new_transaction_ins = input
        .iter()
        .map(|txin| {
          let mut witness_buffer = Vec::new();
          let _ = txin.witness.consensus_encode(&mut witness_buffer);
          let mut witness = String::with_capacity(witness_buffer.len() * 2);
          for byte in witness_buffer.into_iter() {
            let _ = write!(&mut witness, "{:02X}", byte);
          }
          NewTransactionIn {
            block_height: height,
            tx_index: tx_index as i32,
            tx_hash: txid.to_string(),
            previous_output_hash: txin.previous_output.txid.to_string(),
            previous_output_vout: BigDecimal::from(txin.previous_output.vout),
            script_sig: txin.script_sig.to_hex_string(),
            script_asm: txin.script_sig.to_asm_string(),
            sequence_number: BigDecimal::from(txin.sequence.0),
            witness,
          }
        })
        .collect();
      transaction_ins.append(&mut new_transaction_ins);
      //Create transaction out for each transaction then push to common vector for whole block
      let mut new_transaction_outs =
        self.index_transaction_output(height, tx_index, txid, output, artifact.as_ref());
      transaction_outs.append(&mut new_transaction_outs);
    }
    let index_block = match self.get_block_cache(height as u64) {
      Some(cache) => cache,
      None => {
        let new_index_block = Arc::new(IndexBlock::new(height as u64));
        self.add_index_block(Arc::clone(&new_index_block));
        new_index_block
      }
    };
    index_block.add_block(new_block);
    index_block.append_transactions(&mut transactions);
    index_block.append_transaction_ins(&mut transaction_ins);
    index_block.append_transaction_outs(&mut transaction_outs);
    index_block.append_raw_ins(&mut tx_ins);
    Ok(())
  }
  pub fn index_transaction_output(
    &self,
    height: i64,
    tx_index: usize,
    txid: &Txid,
    output: &Vec<TxOut>,
    artifact: Option<&Artifact>,
  ) -> Vec<NewTransactionOut> {
    let new_transaction_outs = output
      .iter()
      .enumerate()
      .map(|(index, tx_out)| {
        let address = Address::from_script(&tx_out.script_pubkey, self.chain.network()).ok();
        let address = address.map(|addr| addr.to_string());

        let asm = tx_out.script_pubkey.to_asm_string();
        let dust_value = tx_out.script_pubkey.dust_value().to_sat() as i64;
        // Index op_return
        // payload starts with OP_RETURN
        // followed by the protocol identifier, ignoring errors, since OP_RETURN
        // scripts may be invalid
        let mut instructions = tx_out.script_pubkey.instructions();
        let mut runestone: Option<String> = None;
        let mut cenotaph: Option<String> = None;
        let mut edicts: i64 = 0;
        let mut burn = false;
        let mut etching = false;
        let mut mint = false;
        if instructions.next() == Some(Ok(Instruction::Op(opcodes::all::OP_RETURN)))
          && instructions.next() == Some(Ok(Instruction::Op(Runestone::MAGIC_NUMBER)))
          && artifact.is_some()
        {
          // construct the payload by concatenating remaining data pushes
          match artifact {
            Some(Artifact::Runestone(rs)) => {
              runestone = serde_json::to_string(&RunestoneValue::from(rs)).ok();
              edicts = rs.edicts.len() as i64;
              etching = rs.etching.is_some();
              mint = rs.mint.is_some();
            }
            Some(Artifact::Cenotaph(v)) => {
              cenotaph = serde_json::to_string(&CenotaphValue::from(v)).ok();
              etching = v.etching.is_some();
              mint = v.mint.is_some();
              burn = true;
            }
            None => {}
          };
        }

        NewTransactionOut {
          block_height: height,
          tx_index: tx_index as i32,
          txout_id: format!("{}:{}", txid, index),
          tx_hash: txid.to_string(),
          vout: BigDecimal::from(index as u64),
          value: BigDecimal::from(tx_out.value),
          address,
          asm,
          dust_value: BigDecimal::from(dust_value),
          script_pubkey: tx_out.script_pubkey.to_hex_string(),
          spent: false,
          runestone: runestone.unwrap_or_else(|| "{}".to_string()),
          cenotaph: cenotaph.unwrap_or_else(|| "{}".to_string()),
          edicts,
          mint,
          etching,
          burn,
        }
      })
      .collect();
    new_transaction_outs
  }

  pub fn index_transaction_rune_entry(
    &self,
    txid: &Txid,
    rune_id: &RuneId,
    rune_entry: &RuneEntry,
  ) -> Result<(), diesel::result::Error> {
    log::debug!("Runebeta index transaction rune {}, rune {}", txid, rune_id);
    let tx_rune_entry = NewTxRuneEntry {
      tx_hash: txid.to_string(),
      block_height: rune_id.block as i64,
      tx_index: rune_id.tx as i32,
      rune_id: rune_id.to_string(),
      burned: BigDecimal::from(rune_entry.burned),
      divisibility: rune_entry.divisibility as i16,
      etching: rune_entry.etching.to_string(),
      parent: None,
      mints: rune_entry.mints as i64,
      number: rune_entry.block as i64,
      rune: BigDecimal::from(rune_entry.spaced_rune.rune.0),
      spacers: rune_entry.spaced_rune.spacers as i32,
      premine: rune_entry.premine as i64,
      spaced_rune: rune_entry.spaced_rune.to_string(),
      supply: BigDecimal::from(0_u128),
      symbol: rune_entry.symbol.map(|c| c.to_string()),
      timestamp: rune_entry.timestamp as i32,
      mint_entry: rune_entry
        .terms
        .map(|entry| MintEntryType::from(&entry))
        .unwrap_or_default(),
      turbo: rune_entry.turbo,
    };
    let height = rune_id.block.clone();
    let index_block = match self.get_block_cache(height as u64) {
      Some(cache) => cache,
      None => {
        let new_index_block = Arc::new(IndexBlock::new(height as u64));
        self.add_index_block(Arc::clone(&new_index_block));
        new_index_block
      }
    };
    index_block.add_rune_entry(tx_rune_entry);
    Ok(())
  }

  pub fn index_outpoint_balances(
    &self,
    block_height: i64,
    tx_index: u32,
    transaction: &Transaction,
    vout: usize,
    balances: &Vec<(RuneId, BigDecimal)>,
  ) -> Result<usize, diesel::result::Error> {
    let mut len = 0;
    let network = self.chain.network();
    let mut tx_runes = vec![];
    let mut outpoint_balances = vec![];
    balances.iter().for_each(|(rune_id, balance)| {
      let txid = transaction.txid();
      if let Some(tx_out) = transaction.output.get(vout) {
        let address = Address::from_script(&tx_out.script_pubkey, network.clone()).ok();
        let address = address.map(|addr| addr.to_string());
        let address = address.map(|addr| addr.to_string()).unwrap_or_default();
        let tx_rune = NewTransactionRune {
          block_height,
          tx_index: tx_index as i32,
          tx_hash: txid.to_string(),
          rune_id: rune_id.to_string(),
        };
        tx_runes.push(tx_rune);
        let outpoint_balance = NewOutpointRuneBalance {
          block_height,
          tx_index: tx_index as i32,
          txout_id: format!("{}:{}", &txid, vout),
          tx_hash: txid.to_string(),
          vout: vout as i64,
          rune_id: rune_id.to_string(),
          address,
          spent: false,
          balance_value: balance.clone(),
        };
        len = len + 1;
        outpoint_balances.push(outpoint_balance);
      }
    });
    let index_block = match self.get_block_cache(block_height as u64) {
      Some(cache) => cache,
      None => {
        let new_index_block = Arc::new(IndexBlock::new(block_height as u64));
        self.add_index_block(Arc::clone(&new_index_block));
        new_index_block
      }
    };
    index_block.append_tx_rune(&mut tx_runes);
    index_block.append_outpoint_rune_balances(&mut outpoint_balances);
    Ok(len)
  }
  pub fn commit(&self) -> anyhow::Result<Vec<JoinHandle<()>>> {
    let mut handles = vec![];
    let mut processing_cache = vec![];
    if let Ok(mut cache) = self.index_cache.write() {
      while let Some(index_block) = cache.pop_front() {
        processing_cache.push(index_block);
      }
    }
    if processing_cache.len() > 0 {
      //let mut connection = self.get_connection().map_err(|err| anyhow!(err))?;
      //Sucess establist db connection
      let table_block = BlockTable::new();
      let table_tranction = TransactionTable::new();
      let table_transaction_in = TransactionInTable::new();
      let table_transaction_out = TransactionOutTable::new();

      let table_outpoint_balance = OutpointRuneBalanceTable::new();
      let table_transaction_rune = TransactionRuneTable::new();
      let table_transaction_rune_entry = TransactionRuneEntryTable::new();

      //Insert records in transactional manner for each block
      //self.index_cache.iter().for_each(transactional_insert);
      //Insert all records without transactional
      //Insert into blocks
      let mut total_blocks = vec![];
      let mut total_transactions = vec![];
      let mut total_transaction_ins = vec![];
      let mut total_transaction_outs = vec![];
      let mut total_outpoint_balances = vec![];
      let mut total_tx_runes = vec![];
      let mut total_rune_entries = vec![];
      let mut total_tx_ins = vec![];
      for index_block in processing_cache {
        let Some(IndexBlock {
          block_height: _block_height,
          raw_tx_ins,
          blocks,
          transactions,
          transaction_ins,
          transaction_outs,
          outpoint_balances,
          tx_runes,
          rune_entries,
        }) = Arc::try_unwrap(index_block).ok()
        else {
          break;
        };
        if let Ok(ref mut blocks) = blocks.into_inner() {
          total_blocks.append(blocks);
        }
        if let Ok(ref mut transactions) = transactions.into_inner() {
          total_transactions.append(transactions);
        }
        if let Ok(ref mut transaction_ins) = transaction_ins.into_inner() {
          total_transaction_ins.append(transaction_ins);
        }
        if let Ok(ref mut transaction_outs) = transaction_outs.into_inner() {
          total_transaction_outs.append(transaction_outs);
        }
        if let Ok(ref mut outpoint_balances) = outpoint_balances.into_inner() {
          total_outpoint_balances.append(outpoint_balances);
        }
        if let Ok(ref mut rune_entries) = rune_entries.into_inner() {
          total_rune_entries.append(rune_entries);
        }
        if let Ok(ref mut tx_runes) = tx_runes.into_inner() {
          total_tx_runes.append(tx_runes);
        }
        if let Ok(ref mut raw_tx_ins) = raw_tx_ins.into_inner() {
          total_tx_ins.append(raw_tx_ins);
        }
      }
      if let Ok(ref mut res) = table_block.insert_vector(total_blocks, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_tranction.insert_vector(total_transactions, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_transaction_in.insert_vector(total_transaction_ins, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_transaction_out.insert_vector(total_transaction_outs, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_outpoint_balance.insert_vector(total_outpoint_balances, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_transaction_rune.insert_vector(total_tx_runes, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if let Ok(ref mut res) =
        table_transaction_rune_entry.insert_vector(total_rune_entries, self.connection_pool.clone())
      {
        handles.append(res);
      }
      if total_tx_ins.len() > 0 {
        if let Ok(ref mut res) =
          table_outpoint_balance.spends(&total_tx_ins, self.connection_pool.clone())
        {
          handles.append(res);
        }
        if let Ok(ref mut res) =
          table_transaction_out.spends(&total_tx_ins, self.connection_pool.clone())
        {
          handles.append(res);
        }
      }
    }
    Ok(handles)
  }
}
