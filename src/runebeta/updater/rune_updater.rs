use {
  self::{
    entry::Entry,
    storage::{OutpointRuneBalaneTable, RuneTable, SequenceNumberRuneIdTable, TxidRuneTable},
  },
  super::*,
  crate::{
    runebeta::{
      connection::WriteTransaction,
      models::{NewOutpointRuneBalance, NewRune, NewRuneEntries, OutpointRuneBalance, U128},
    },
    runes::{self, varint, Edict, Runestone},
    InscriptionId, RuneId,
  },
};

struct Claim {
  id: RuneId,
  limit: u128,
}

struct Etched {
  balance: u128,
  divisibility: u8,
  id: RuneId,
  mint: Option<MintEntry>,
  rune: Rune,
  spacers: u32,
  symbol: Option<char>,
}

#[derive(Default)]
pub(crate) struct RuneUpdate {
  pub(crate) burned: u128,
  pub(crate) mints: u64,
  pub(crate) supply: u128,
}

pub(super) struct RuneUpdater<'db> {
  pub(super) height: u32,
  //pub(super) id_to_entry: &'a mut Table<'db, 'tx, RuneIdValue, RuneEntryValue>,
  pub(super) id_to_entry: RuneEntryTable<'db>,
  //Mapping giua sequence number, tx_hash, tx_height and rune_index
  pub(super) sequence_number_runeid: SequenceNumberRuneIdTable<'db>,
  //pub(super) inscription_id_to_sequence_number: &'a Table<'db, 'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Rune,
  //pub(super) outpoint_to_balances: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) outpoint_to_balances: OutpointRuneBalaneTable<'db>,
  pub(super) rune_to_id: RuneTable<'db>,
  pub(super) runes: u64,
  //pub(super) sequence_number_to_rune_id: &'a mut Table<'db, 'tx, u32, RuneIdValue>,
  pub(super) statistic_table: StatisticTable<'db>,
  pub(super) timestamp: u32,
  pub(super) transaction_id_to_rune: TxidRuneTable<'db>,
  pub(super) updates: HashMap<RuneId, RuneUpdate>,
}

impl<'a, 'db, 'tx> RuneUpdater<'db> {
  pub fn new(
    height: u32,
    minimum: Rune,
    runes: u64,
    timestamp: u32,
    wtx: &'db mut WriteTransaction,
  ) -> Self {
    Self {
      height,
      id_to_entry: wtx.create_rune_entry_table(),
      sequence_number_runeid: wtx.create_sequence_number_runeid_table(),
      //inscription_id_to_sequence_number: wtx.open_table(table),
      minimum,
      outpoint_to_balances: wtx.create_outpoint_rune_balance_table(),
      rune_to_id: wtx.create_rune_table(),
      runes,
      //sequence_number_to_rune_id: todo!(),
      statistic_table: StatisticTable::new(wtx.get_connection()),
      timestamp,
      transaction_id_to_rune: wtx.create_txid_rune_table(),
      updates: HashMap::new(),
    }
  }
  pub(super) fn index_runes(
    &mut self,
    tx_index: usize,
    tx: &Transaction,
    txid: Txid,
  ) -> Result<()> {
    let runestone = Runestone::from_transaction(tx);

    let mut unallocated = self.unallocated(tx)?;

    let cenotaph = runestone
      .as_ref()
      .map(|runestone| runestone.cenotaph)
      .unwrap_or_default();

    let default_output = runestone.as_ref().and_then(|runestone| {
      runestone
        .default_output
        .and_then(|default| usize::try_from(default).ok())
    });

    let mut allocated: Vec<HashMap<RuneId, u128>> = vec![HashMap::new(); tx.output.len()];

    if let Some(runestone) = runestone {
      if let Some(claim) = runestone
        .claim
        .and_then(|id| self.claim(id).transpose())
        .transpose()?
      {
        *unallocated.entry(claim.id).or_default() += claim.limit;

        let update = self.updates.entry(claim.id).or_default();

        update.mints += 1;
        update.supply += claim.limit;
      }

      let mut etched = self.etched(tx_index, tx, &runestone)?;

      if !cenotaph {
        for Edict { id, amount, output } in runestone.edicts {
          let Ok(output) = usize::try_from(output) else {
            continue;
          };

          // edicts with output values greater than the number of outputs
          // should never be produced by the edict parser
          assert!(output <= tx.output.len());

          let (balance, id) = if id == RuneId::default() {
            // If this edict allocates new issuance runes, skip it
            // if no issuance was present, or if the issuance was invalid.
            // Additionally, replace ID 0 with the newly assigned ID, and
            // get the unallocated balance of the issuance.
            match etched.as_mut() {
              Some(Etched { balance, id, .. }) => (balance, *id),
              None => continue,
            }
          } else {
            // Get the unallocated balance of the given ID
            match unallocated.get_mut(&id) {
              Some(balance) => (balance, id),
              None => continue,
            }
          };

          let mut allocate = |balance: &mut u128, amount: u128, output: usize| {
            if amount > 0 {
              *balance -= amount;
              *allocated[output].entry(id).or_default() += amount;
            }
          };

          if output == tx.output.len() {
            // find non-OP_RETURN outputs
            let destinations = tx
              .output
              .iter()
              .enumerate()
              .filter_map(|(output, tx_out)| {
                (!tx_out.script_pubkey.is_op_return()).then_some(output)
              })
              .collect::<Vec<usize>>();

            if amount == 0 {
              // if amount is zero, divide balance between eligible outputs
              let amount = *balance / destinations.len() as u128;
              let remainder = usize::try_from(*balance % destinations.len() as u128).unwrap();

              for (i, output) in destinations.iter().enumerate() {
                allocate(
                  balance,
                  if i < remainder { amount + 1 } else { amount },
                  *output,
                );
              }
            } else {
              // if amount is non-zero, distribute amount to eligible outputs
              for output in destinations {
                allocate(balance, amount.min(*balance), output);
              }
            }
          } else {
            // Get the allocatable amount
            let amount = if amount == 0 {
              *balance
            } else {
              amount.min(*balance)
            };

            allocate(balance, amount, output);
          }
        }
      }

      if let Some(etched) = etched {
        self.create_rune_entry(txid, cenotaph, etched)?;
      }
    }

    let mut burned: HashMap<RuneId, u128> = HashMap::new();

    if cenotaph {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      // assign all un-allocated runes to the default output, or the first non
      // OP_RETURN output if there is no default, or if the default output is
      // too large
      if let Some(vout) = default_output
        .filter(|vout| *vout < allocated.len())
        .or_else(|| {
          tx.output
            .iter()
            .enumerate()
            .find(|(_vout, tx_out)| !tx_out.script_pubkey.is_op_return())
            .map(|(vout, _tx_out)| vout)
        })
      {
        for (id, balance) in unallocated {
          if balance > 0 {
            *allocated[vout].entry(id).or_default() += balance;
          }
        }
      } else {
        for (id, balance) in unallocated {
          if balance > 0 {
            *burned.entry(id).or_default() += balance;
          }
        }
      }
    }

    // update outpoint balances
    let mut outpoint_balances: Vec<NewOutpointRuneBalance> = Vec::new();
    //let mut buffer: Vec<u8> = Vec::new();
    for (vout, balances) in allocated.into_iter().enumerate() {
      if balances.is_empty() {
        continue;
      }

      // increment burned balances
      if tx.output[vout].script_pubkey.is_op_return() {
        for (id, balance) in &balances {
          *burned.entry(*id).or_default() += balance;
        }
        continue;
      }

      //buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(RuneId, u128)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      for (id, balance) in balances {
        // varint::encode_to_vec(id, &mut buffer);
        // varint::encode_to_vec(balance, &mut buffer);
        let new_outpoint_rune_balance = NewOutpointRuneBalance {
          tx_hash: txid.to_string().as_str(),
          vout: vout as i32,
          rune_block: id.block as i32,
          rune_tx: id.tx as i16,
          balance_value: &U128(balance),
        };
        outpoint_balances.push(new_outpoint_rune_balance);
      }

      // self.outpoint_to_balances.insert(
      //   &OutPoint {
      //     txid,
      //     vout: vout.try_into().unwrap(),
      //   }
      //   .store(),
      //   buffer.as_slice(),
      // )?;
    }
    self.outpoint_to_balances.inserts(&outpoint_balances)?;

    for input in tx.input.iter() {
      if input.previous_output.is_null() {
        continue;
      }

      self
        .outpoint_to_output
        .remove(&input.previous_output.store())?
        .unwrap();
    }

    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        txid,
        vout: vout.try_into().unwrap(),
      };

      self.outpoint_to_output.insert(
        &outpoint.store(),
        OutputEntry {
          height: self.height,
          taproot: output.script_pubkey.is_v1_p2tr(),
        }
        .store(),
      )?;
    }

    // increment entries with burned runes
    for (id, amount) in burned {
      self.updates.entry(id).or_default().burned += amount;
    }

    Ok(())
  }

  fn create_rune_entry(&mut self, txid: Txid, burn: bool, etched: Etched) -> Result {
    let Etched {
      balance,
      divisibility,
      id,
      mint,
      rune,
      spacers,
      symbol,
    } = etched;

    let new_rune = NewRune {
      rune: U128(rune.0),
      tx_height: id.block as i64,
      rune_index: id.tx as i16,
    };
    self.rune_to_id.insert(&new_rune)?;
    //self.transaction_id_to_rune.insert(&txid.store(), rune.0)?;
    self.transaction_id_to_rune.insert(&txid, U128(rune.0))?;
    let number = self.runes;
    self.runes += 1;
    let premine = u128::MAX - balance;
    // self
    //   .statistic_to_count
    //   .insert(&Statistic::Runes.into(), self.runes)?;
    self.statistic_table.set_runes(self.runes as i64)?;

    self.id_to_entry.insert(&NewRuneEntries {
      rune_height: id.block as i32,
      rune_index: id.tx as i16,
      burned: U128(0),
      divisibility: divisibility.into(),
      etching: txid.to_string().as_str(),
      mints: 0,
      number: number.try_into().unwrap(),
      mint: mint.and_then(|mint| (!burn).then_some(mint)),
      rune: U128(rune.0),
      spacers: spacers.try_into().unwrap(),
      supply: U128(premine),
      symbol: symbol.as_ref(),
      timestamp: self.timestamp as i32,
    })?;

    let inscription_id = InscriptionId { txid, index: 0 };

    if let Some(sequence_number) = self
      .inscription_id_to_sequence_number
      .get(&inscription_id.store())?
    {
      self
        .sequence_number_to_rune_id
        .insert(sequence_number.value(), id.store())?;
    }

    Ok(())
  }

  fn etched(
    &mut self,
    tx_index: usize,
    tx: &Transaction,
    runestone: &Runestone,
  ) -> Result<Option<Etched>> {
    let Some(etching) = runestone.etching else {
      return Ok(None);
    };

    let rune = if let Some(rune) = etching.rune {
      if rune < self.minimum
        || rune.is_reserved()
        || self.rune_to_id.get(rune.0)?.is_some()
        || !self.tx_commits_to_rune(tx, rune)?
      {
        return Ok(None);
      }
      rune
    } else {
      let reserved_runes = self
        .statistic_table
        .get_reserved_runes()
        .unwrap_or_default();
      self
        .statistic_table
        .set_reserved_runes(reserved_runes + 1)?;
      Rune::reserved((reserved_runes as u64).into())
    };

    // Nota bene: Because it would require constructing a block
    // with 2**16 + 1 transactions, there is no test that checks that
    // an eching in a transaction with an out-of-bounds index is
    // ignored.
    let Ok(index) = u16::try_from(tx_index) else {
      return Ok(None);
    };

    Ok(Some(Etched {
      balance: u128::MAX,
      divisibility: etching.divisibility,
      id: RuneId {
        block: self.height,
        tx: index,
      },
      rune,
      spacers: etching.spacers,
      symbol: etching.symbol,
      mint: etching.mint.map(|mint| MintEntry {
        deadline: mint.deadline,
        end: mint.term.map(|term| term + self.height),
        limit: mint.limit.map(|limit| limit.min(runes::MAX_LIMIT)),
      }),
    }))
  }

  fn claim(&self, id: RuneId) -> Result<Option<Claim>> {
    let Ok(key) = RuneId::try_from(id) else {
      return Ok(None);
    };

    let Some(entry) = self.id_to_entry.get(&key)? else {
      return Ok(None);
    };

    //let entry = RuneEntry::load(entry.value());

    let Some(mint) = entry.mint else {
      return Ok(None);
    };

    if let Some(end) = mint.end {
      if self.height >= end {
        return Ok(None);
      }
    }

    if let Some(deadline) = mint.deadline {
      if self.timestamp >= deadline {
        return Ok(None);
      }
    }

    Ok(Some(Claim {
      id,
      limit: mint.limit.unwrap_or(runes::MAX_LIMIT),
    }))
  }

  fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<RuneId, u128>> {
    // map of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<RuneId, u128> = HashMap::new();

    // increment unallocated runes with the runes in tx inputs
    for input in &tx.input {
      let rune_balances = self.outpoint_to_balances.remove(&input.previous_output)?;
      for OutpointRuneBalance {
        id,
        tx_hash,
        vout,
        balance_id,
        balance_value,
      } in rune_balances.into_iter()
      {
        *unallocated
          .entry(balance_id.try_into().unwrap())
          .or_default() += balance_value;
      }
    }

    Ok(unallocated)
  }
}
