use crate::{
  runebeta::{
    models::{MintEntryType, NewOutpointRuneBalance},
    OutpointRuneBalanceTable,
  },
  Chain, RuneEntry, RuneId,
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
use std::fmt::Write;
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
  index_cache: Vec<IndexBlock>,
}
impl IndexExtension {
  pub fn new(chain: Chain) -> Self {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    //Rune db migration
    loop {
      log::info!("Try to connection to db");
      let Ok(mut connection) = PgConnection::establish(&database_url) else {
        let ten_second = time::Duration::from_secs(10);
        thread::sleep(ten_second);
        continue;
      };
      let mut harness_with_output = HarnessWithOutput::write_to_stdout(&mut connection);
      let res = harness_with_output.run_pending_migrations(MIGRATIONS);
      log::info!("Run migration with result {:?}", &res);
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
            previous_output_vout: txin.previous_output.vout as i64,
            script_sig: txin.script_sig.to_hex_string(),
            sequence_number: txin.sequence.0 as i64,
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
    self.index_cache.push(index_block);
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
        let mut runestone: Option<RunestoneValue> = None;
        let mut cenotaph: Option<CenotaphValue> = None;
        let mut edicts: i32 = 0;
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
              runestone = Some(RunestoneValue::from(rs));
              edicts = rs.edicts.len() as i32;
              etching = rs.etching.is_some();
              mint = rs.mint.is_some();
            }
            Some(Artifact::Cenotaph(v)) => {
              cenotaph = Some(CenotaphValue::from(v));
              etching = v.etching.is_some();
              mint = v.mint.is_some();
              burn = true;
            }
            None => {}
          };
        }

        NewTransactionOut {
          tx_hash: txid.to_string(),
          vout: index as i64,
          value: tx_out.value as i64,
          address,
          asm,
          dust_value,
          script_pubkey: tx_out.script_pubkey.to_hex_string(),
          spent: false,
          runestone: runestone.unwrap_or_default(),
          cenotaph: cenotaph.unwrap_or_default(),
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
        self.index_cache.push(IndexBlock::new(height as u64));
        self.index_cache.last_mut().unwrap()
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
    log::info!("Runebeta index outpoint balances of transaction {}", txid);
    let mut len = 0;
    balances.iter().for_each(|(rune_id, balance)| {
      let height = rune_id.block.clone();
      let index_block = match self.get_block_cache(height as u64) {
        Some(cache) => cache,
        None => {
          self.index_cache.push(IndexBlock::new(height));
          self.index_cache.last_mut().unwrap()
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
    log::info!("Commit cache for {} blocks", len);
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
        self.index_cache.iter().for_each(|cache| {
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
              if cache.blocks.len() > 0 {
                let chunk_size = (u16::MAX / super::table_block::NUMBER_OF_FIELDS) as usize;
                let chunks = cache.blocks.chunks(chunk_size);
                for chunk in chunks {
                  let res = table_block.inserts(chunk, conn);
                  if res.is_err() {
                    log::info!("Insert blocks error {:?}", &res);
                    res?;
                  }
                }
              }
              if cache.transactions.len() > 0 {
                let chunk_size = (u16::MAX / super::table_transaction::NUMBER_OF_FIELDS) as usize;
                let chunks = cache.transactions.chunks(chunk_size);
                for chunk in chunks {
                  let res = table_tranction.inserts(chunk, conn);
                  if res.is_err() {
                    log::info!("Insert transactions error {:?}", &res);
                    res?;
                  }
                }
              }
              if cache.transaction_ins.len() > 0 {
                let chunk_size = u16::MAX / super::table_transaction_in::NUMBER_OF_FIELDS;
                let chunks = cache.transaction_ins.chunks(chunk_size as usize);
                for chunk in chunks {
                  table_transaction_in.inserts(chunk, conn)?;
                }
              }
              if cache.tx_ins.len() > 0 {
                table_transaction_out.spends(&cache.tx_ins, conn)?;
              }
              if cache.transaction_outs.len() > 0 {
                let chunk_size = u16::MAX / super::table_transaction_out::NUMBER_OF_FIELDS;
                let chunks = cache.transaction_outs.chunks(chunk_size as usize);
                for chunk in chunks {
                  table_transaction_out.inserts(chunk, conn)?;
                }
              }
              if cache.outpoint_balances.len() > 0 {
                let chunk_size = u16::MAX / super::table_outpoint_rune_balance::NUMBER_OF_FIELDS;
                let chunks = cache.outpoint_balances.chunks(chunk_size as usize);
                for chunk in chunks {
                  table_outpoint_balance.insert(chunk, conn)?;
                }
              }
              if cache.rune_entries.len() > 0 {
                let chunk_size = u16::MAX / super::table_transaction_rune_entry::NUMBER_OF_FIELDS;
                let chunks = cache.rune_entries.chunks(chunk_size as usize);
                for chunk in chunks {
                  table_tranction_rune.inserts(chunk, conn)?;
                }
              }
              Ok(())
            });
          if res.is_err() {
            log::info!("Transaction index result {:?}", &res);
            //panic!("Transaction index result {:?}", &res);
          }
        });
        self.index_cache.clear();
        break;
      }
    }
    Ok(len)
  }
}
