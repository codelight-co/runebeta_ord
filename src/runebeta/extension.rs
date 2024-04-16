use crate::{
  runebeta::{
    models::{MintEntryType, NewOutpointRuneBalance},
    OutpointRuneBalanceTable,
  },
  Chain, InsertRecords, RuneEntry, RuneId,
};

use super::{
  models::{
    CenotaphValue, NewBlock, NewTransaction, NewTransactionIn, NewTransactionOut, NewTxRuneEntry,
    RunestoneValue,
  },
  table_transaction::TransactionTable,
  BlockTable, TransactionInTable, TransactionOutTable, TransactionRuneEntryTable,
};
use bigdecimal::BigDecimal;
use bitcoin::{
  block::Header, consensus::Encodable, opcodes, script::Instruction, Address, Transaction, TxOut,
  Txid,
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{
  embed_migrations, EmbeddedMigrations, HarnessWithOutput, MigrationHarness,
};
use dotenvy::dotenv;
use ordinals::{Artifact, Runestone};
use std::{collections::VecDeque, fmt::Write, time::Instant};
use std::{env, thread, time};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Default)]
pub struct IndexBlock {
  block_height: u64,
  // Raw txin for update previous txouts'spent => true
  tx_ins: Vec<(Txid, i64)>,
  blocks: Vec<NewBlock>,
  transactions: Vec<NewTransaction>,
  transaction_ins: Vec<NewTransactionIn>,
  transaction_outs: Vec<NewTransactionOut>,
  //Store outpoint banlance in each rune update
  outpoint_balances: Vec<NewOutpointRuneBalance>,
  rune_entries: Vec<NewTxRuneEntry>,
}
impl IndexBlock {
  pub fn new(height: u64) -> Self {
    Self {
      block_height: height,
      ..Default::default()
    }
  }
  pub fn get_size(&self) -> u128 {
    self.blocks.len() as u128
      + self.transactions.len() as u128
      + self.transaction_ins.len() as u128
      + self.transaction_outs.len() as u128
      + self.outpoint_balances.len() as u128
      + self.rune_entries.len() as u128
      + self.tx_ins.len() as u128
  }
  fn _clear(&mut self) {
    self.blocks.clear();
    self.transactions.clear();
    self.transaction_ins.clear();
    self.transaction_outs.clear();
    self.tx_ins.clear();
    self.rune_entries.clear();
    self.outpoint_balances.clear();
  }
}
#[derive(Debug)]
pub struct IndexExtension {
  chain: Chain,
  database_url: String,
  index_cache: VecDeque<IndexBlock>,
}
impl IndexExtension {
  pub fn new(chain: Chain) -> Self {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //Rune db migration
    loop {
      log::debug!("Try to connection to db");
      let Ok(mut connection) = PgConnection::establish(&database_url) else {
        let ten_second = time::Duration::from_secs(10);
        thread::sleep(ten_second);
        continue;
      };
      let mut harness_with_output = HarnessWithOutput::write_to_stdout(&mut connection);
      let res = harness_with_output.run_pending_migrations(MIGRATIONS);
      if res.is_err() {
        log::info!("Run migration with error {:?}", &res);
      }
      break;
    }
    Self {
      chain,
      database_url,
      index_cache: Default::default(),
    }
  }

  /*
   * Loop until successfull establish connection
   */
  pub fn get_connection(&self) -> Option<PgConnection> {
    loop {
      let Ok(connection) = PgConnection::establish(&self.database_url) else {
        let ten_second = time::Duration::from_secs(10);
        thread::sleep(ten_second);
        continue;
      };
      return Some(connection);
    }
  }
  ///
  ///
  pub fn get_block_cache(&mut self, height: u64) -> Option<&mut IndexBlock> {
    for cache in self.index_cache.iter_mut() {
      if cache.block_height == height {
        //get existing cache
        return Some(cache);
      }
    }
    None
  }
  // pub fn get_indexed_block_height(&self) -> Result<i64, diesel::result::Error> {
  //   if let Some(mut connection) = self.get_connection() {
  //     let table_block = BlockTable::new();
  //     table_block.get_latest_block_height(&mut connection)
  //   } else {
  //     log::debug!("Cannot establish connection");
  //     Ok(0)
  //   }
  // }
  pub fn index_block(
    &mut self,
    height: i64,
    block_header: &Header,
    block_data: &Vec<(Transaction, Txid)>,
  ) -> Result<(), diesel::result::Error> {
    log::info!("Index block {}", &height);
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

    for (tx, txid) in block_data.iter() {
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
        lock_time: lock_time.to_consensus_u32() as i64,
        tx_hash: txid.to_string(),
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
      let mut new_transaction_outs = self.index_transaction_output(txid, output, artifact.as_ref());
      transaction_outs.append(&mut new_transaction_outs);
    }
    let index_block = IndexBlock {
      block_height: height as u64,
      tx_ins,
      blocks: vec![new_block],
      transactions,
      transaction_ins,
      transaction_outs,
      ..Default::default()
    };
    self.index_cache.push_back(index_block);
    Ok(())
  }
  pub fn index_transaction_output(
    &self,
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
          txout_id: format!("{}:{}", txid, index),
          tx_hash: txid.to_string(),
          vout: BigDecimal::from(index as u64),
          value: BigDecimal::from(tx_out.value),
          address,
          asm,
          dust_value: BigDecimal::from(dust_value),
          script_pubkey: tx_out.script_pubkey.to_hex_string(),
          spent: false,
          runestone: runestone.unwrap_or_else(||"{}".to_string()),
          cenotaph: cenotaph.unwrap_or_else(||"{}".to_string()),
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
    &mut self,
    txid: &Txid,
    rune_id: &RuneId,
    rune_entry: &RuneEntry,
  ) -> Result<(), diesel::result::Error> {
    log::info!("Runebeta index transaction rune {}, rune {}", txid, rune_id);
    let tx_rune_entry = NewTxRuneEntry {
      tx_hash: txid.to_string(),
      // rune_height: rune_id.block as i32,
      // rune_index: rune_id.tx as i16,
      rune_id: rune_id.to_string(),
      burned: BigDecimal::from(rune_entry.burned),
      divisibility: rune_entry.divisibility as i16,
      etching: rune_entry.etching.to_string(),
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
        self.index_cache.push_back(IndexBlock::new(height as u64));
        self.index_cache.back_mut().unwrap()
      }
    };
    index_block.rune_entries.push(tx_rune_entry);
    Ok(())
  }

  pub fn index_outpoint_balances(
    &mut self,
    txid: &Txid,
    vout: i64,
    balances: &Vec<(RuneId, BigDecimal)>,
  ) -> Result<usize, diesel::result::Error> {
    let mut len = 0;
    balances.iter().for_each(|(rune_id, balance)| {
      let height = rune_id.block.clone();
      let index_block = match self.get_block_cache(height as u64) {
        Some(cache) => cache,
        None => {
          self.index_cache.push_back(IndexBlock::new(height));
          self.index_cache.back_mut().unwrap()
        }
      };
      let outpoint_balance = NewOutpointRuneBalance {
        tx_hash: txid.to_string(),
        vout,
        rune_id: rune_id.to_string(),
        balance_value: balance.clone(),
      };
      len = len + 1;
      index_block.outpoint_balances.push(outpoint_balance);
    });
    Ok(len)
  }
  pub fn commit(&mut self) -> anyhow::Result<usize> {
    let len = self.index_cache.len();
    // log::info!("Commit cache for {} blocks", len);
    if len > 0 {
      //Try connect to postgres db until successfully establish connection
      loop {
        let Ok(mut connection) = PgConnection::establish(&self.database_url) else {
          let ten_second = time::Duration::from_secs(10);
          thread::sleep(ten_second);
          continue;
        };
        //Sucess establist db connection
        let table_block = BlockTable::new();
        let table_tranction = TransactionTable::new();
        let table_transaction_in = TransactionInTable::new();
        let table_transaction_out = TransactionOutTable::new();

        let table_outpoint_balance = OutpointRuneBalanceTable::new();
        let table_tranction_rune = TransactionRuneEntryTable::new();
        let _transactional_insert = |cache: &IndexBlock| {
          let res: Result<(), diesel::result::Error> =
            connection.build_transaction().read_write().run(|conn| {
              log::info!(
                "Commit cache {} with {} blocks, {} txs, {} txins, {} txouts, {} outpoint_balances, {} rune entries",
                cache.block_height,
                cache.blocks.len(),
                cache.transactions.len(),
                cache.transaction_ins.len(),
                cache.transaction_outs.len(),
                cache.outpoint_balances.len(),
                cache.rune_entries.len()
              );
              //must be safe to unwrap;
              let res = table_block.insert_vector(&cache.blocks, conn);
              if res.is_err() {
                log::info!("Insert blocks error {:?}", &res);
                res?;
              }
              let res = table_tranction.insert_vector(&cache.transactions, conn);
              if res.is_err() {
                log::info!("Insert transactions error {:?}", &res);
                res?;
              }
              let res = table_transaction_in.insert_vector(&cache.transaction_ins, conn);
              if res.is_err() {
                log::info!("Insert transaction ins error {:?}", &res);
                res?;
              }
              let res = table_transaction_out.insert_vector(&cache.transaction_outs, conn);
              if res.is_err() {
                log::info!("Insert transaction outs error {:?}", &res);
                res?;
              }

              if cache.tx_ins.len() > 0 {
                table_transaction_out.spends(&cache.tx_ins, conn)?;
              }

              let res = table_outpoint_balance.insert_vector(&cache.outpoint_balances, conn);
              if res.is_err() {
                log::info!("Insert outpoint balances error {:?}", &res);
                res?;
              }
              
              let res = table_tranction_rune.insert_vector(&cache.rune_entries, conn);
              if res.is_err() {
                log::info!("Insert rune entries error {:?}", &res);
                res?;
              }
              Ok(())
            });
          if res.is_err() {
            log::info!("Transaction index result {:?}", &res);
            //panic!("Transaction index result {:?}", &res);
          }
        };
        //Insert records in transactional manner for each block
        //self.index_cache.iter().for_each(transactional_insert);
        //Insert all records without transactional
        //Insert into blocks
        let mut total_blocks = vec![];
        let mut total_transactions = vec![];
        let mut total_transaction_ins = vec![];
        let mut total_transaction_outs = vec![];
        let mut total_outpoint_balances = vec![];
        let mut total_transaction_runes = vec![];
        let mut total_tx_ins = vec![];
        loop {
          let Some(IndexBlock {
            block_height: _,
            tx_ins,
            blocks,
            transactions,
            transaction_ins,
            transaction_outs,
            outpoint_balances,
            rune_entries,
          }) = self.index_cache.pop_front()
          else {
            break;
          };
          total_blocks.extend(blocks);
          total_transactions.extend(transactions);
          total_transaction_ins.extend(transaction_ins);
          total_transaction_outs.extend(transaction_outs);
          total_outpoint_balances.extend(outpoint_balances);
          total_transaction_runes.extend(rune_entries);
          total_tx_ins.extend(tx_ins);
        }
        let mut start = Instant::now();
        let _res = table_block.insert_vector(&total_blocks, &mut connection);
        log::info!(
            "Inserted {} blocks in {} ms", total_blocks.len(), start.elapsed().as_millis());
        start = Instant::now();    
        let _res = table_tranction.insert_vector(&total_transactions, &mut connection);
        log::info!(
            "Inserted {} transactions in {} ms", total_transactions.len(), start.elapsed().as_millis());
        start = Instant::now();
        let _res = table_transaction_in.insert_vector(&total_transaction_ins, &mut connection);
        log::info!(
            "Inserted {} txins in {} ms", total_transaction_ins.len(), start.elapsed().as_millis());
        start = Instant::now();
        let _res = table_transaction_out.insert_vector(&total_transaction_outs, &mut connection);
        log::info!(
            "Inserted {} txouts in {} ms", total_transaction_outs.len(), start.elapsed().as_millis());
        start = Instant::now();
        let _res = table_outpoint_balance.insert_vector(&total_outpoint_balances, &mut connection);
        log::info!(
            "Inserted {} outpoint balances in {} ms", total_outpoint_balances.len(), start.elapsed().as_millis());
        start = Instant::now();
        let _res = table_tranction_rune.insert_vector(&total_transaction_runes, &mut connection);
        log::info!(
            "Inserted {} runes {} ms", total_transaction_runes.len(), start.elapsed().as_millis());
        if total_tx_ins.len() > 0 {
          start = Instant::now();
          table_transaction_out.spends(&total_tx_ins, &mut connection)?;
          log::info!(
            "Update {} spent txout {} ms", total_tx_ins.len(), start.elapsed().as_millis());
        }
        self.index_cache.clear();
        break;
      }
    }
    Ok(len)
  }
}
