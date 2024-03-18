use {
  super::*,
  crate::{
    runebeta::{
      connection::WriteTransaction,
      models::{NewOutpointRuneBalance, NewRune, NewStatistic, NewTxidRune, U128},
    },
    runes::{self, varint, Edict, Runestone},
    schema::{
      self, outpoint_rune_balances, rune_entries, sequence_number_runeid, statistics, txid_rune,
    },
    InscriptionId, RuneId,
  },
};

struct Claim {
  id: u128,
  limit: u128,
}

struct Etched {
  balance: u128,
  divisibility: u8,
  id: u128,
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

pub(super) struct RuneUpdater<'a, 'db, 'tx> {
  pub(super) height: u32,
  //pub(super) id_to_entry: &'a mut Table<'db, 'tx, RuneIdValue, RuneEntryValue>,
  pub(super) id_to_entry: Table<'db, 'tx, rune_entries::table>,
  //Mapping giua sequence number, tx_hash, tx_height and rune_index
  pub(super) sequence_number_runeid: Table<'db, 'tx, sequence_number_runeid::table>,
  //pub(super) inscription_id_to_sequence_number: &'a Table<'db, 'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Rune,
  //pub(super) outpoint_to_balances: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) outpoint_to_balances: Table<'db, 'tx, outpoint_rune_balances::table>,
  pub(super) rune_to_id: Table<'db, 'tx, schema::runes::table>,
  pub(super) runes: u64,
  //pub(super) sequence_number_to_rune_id: &'a mut Table<'db, 'tx, u32, RuneIdValue>,
  pub(super) statistics: Table<'db, 'tx, statistics::table>,
  pub(super) timestamp: u32,
  pub(super) transaction_id_to_rune: Table<'db, 'tx, txid_rune::table>,
  pub(super) updates: HashMap<RuneId, RuneUpdate>,
}

impl<'a, 'db, 'tx> RuneUpdater<'a, 'db, 'tx> {
  pub fn new(
    height: u32,
    minimum: Rune,
    runes: u64,
    timestamp: u32,
    wtx: &'db mut WriteTransaction,
  ) -> Self {
    Self {
      height,
      id_to_entry: wtx.open_table(rune_entries::table::default()),
      sequence_number_runeid: wtx.open_table(sequence_number_runeid::table::default()),
      //inscription_id_to_sequence_number: wtx.open_table(table),
      minimum,
      outpoint_to_balances: wtx.open_table(outpoint_rune_balances::table::default()),
      rune_to_id: wtx.open_table(schema::runes::table::default()),
      runes,
      //sequence_number_to_rune_id: todo!(),
      statistics: wtx.open_table(schema::statistics::table::default()),
      timestamp,
      transaction_id_to_rune: wtx.open_table(txid_rune::table::default()),
      updates: HashMap::new(),
    }
  }
  pub(super) fn index_runes(&mut self, index: usize, tx: &Transaction, txid: Txid) -> Result<()> {
    let runestone = Runestone::from_transaction(tx);

    let mut unallocated = self.unallocated(tx)?;

    let burn = runestone
      .as_ref()
      .map(|runestone| runestone.burn)
      .unwrap_or_default();

    let default_output = runestone.as_ref().and_then(|runestone| {
      runestone
        .default_output
        .and_then(|default| usize::try_from(default).ok())
    });

    let mut allocated: Vec<HashMap<u128, u128>> = vec![HashMap::new(); tx.output.len()];

    if let Some(runestone) = runestone {
      if let Some(claim) = runestone
        .claim
        .and_then(|id| self.claim(id).transpose())
        .transpose()?
      {
        *unallocated.entry(claim.id).or_default() += claim.limit;

        let update = self
          .updates
          .entry(RuneId::try_from(claim.id).unwrap())
          .or_default();

        update.mints += 1;
        update.supply += claim.limit;
      }

      let mut etched = self.etched(index, &runestone)?;

      if !burn {
        for Edict { id, amount, output } in runestone.edicts {
          let Ok(output) = usize::try_from(output) else {
            continue;
          };

          // Skip edicts not referring to valid outputs
          if output > tx.output.len() {
            continue;
          }

          let (balance, id) = if id == 0 {
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
        self.create_rune_entry(txid, burn, etched)?;
      }
    }

    let mut burned: HashMap<u128, u128> = HashMap::new();

    if burn {
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
    let mut buffer: Vec<u8> = Vec::new();
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

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(u128, u128)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      for (id, balance) in balances {
        varint::encode_to_vec(id, &mut buffer);
        varint::encode_to_vec(balance, &mut buffer);
        let new_outpoint_rune_balance = NewOutpointRuneBalance {
          tx_hash: txid.clone(),
          vout,
          balance_id: U128(id),
          balance_value: U128(id),
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
    // increment entries with burned runes
    for (id, amount) in burned {
      self
        .updates
        .entry(RuneId::try_from(id).unwrap())
        .or_default()
        .burned += amount;
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

    let RuneId { height, index } = RuneId::try_from(id).unwrap();
    let new_rune = NewRune {
      rune: rune.0,
      tx_height: height,
      rune_index: index,
    };
    self.rune_to_id.insert(&new_rune)?;
    let new_txid_rune = NewTxidRune {
      tx_hash: txid.to_raw_hash().to_string().as_str(),
      rune: U128(rune.0),
    };
    //self.transaction_id_to_rune.insert(&txid.store(), rune.0)?;
    self.transaction_id_to_rune.insert(&new_txid_rune)?;
    let number = self.runes;
    self.runes += 1;
    // self
    //   .statistic_to_count
    //   .insert(&Statistic::Runes.into(), self.runes)?;
    let new_statistic_rune = NewIndexingStatistic {
      rune_type: Statistic::Runes.into(),
      rune_counter: self.runes,
    };
    self.statistic_to_count.insert(&new_statistic_rune)?;
    self.id_to_entry.insert(
      &RuneEntry {
        burned: 0,
        divisibility,
        etching: txid,
        mints: 0,
        number,
        mint: mint.and_then(|mint| (!burn).then_some(mint)),
        rune,
        spacers,
        supply: if let Some(mint) = mint {
          if mint.end == Some(self.height) {
            0
          } else {
            mint.limit.unwrap_or(runes::MAX_LIMIT)
          }
        } else {
          u128::MAX
        } - balance,
        symbol,
        timestamp: self.timestamp,
      }
      .store(),
    )?;

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

  fn etched(&mut self, index: usize, runestone: &Runestone) -> Result<Option<Etched>> {
    let Some(etching) = runestone.etching else {
      return Ok(None);
    };

    if etching
      .rune
      .map(|rune| rune < self.minimum || rune.is_reserved())
      .unwrap_or_default()
      || etching
        .rune
        .and_then(|rune| self.rune_to_id.get(rune.0).transpose())
        .transpose()?
        .is_some()
    {
      return Ok(None);
    }

    let rune = if let Some(rune) = etching.rune {
      rune
    } else {
      let reserved_runes = self
        .statistics
        .get(&Statistic::ReservedRunes.into())?
        .map(|entry| entry.value())
        .unwrap_or_default();
      let statistic = NewStatistic {
        rune_type: Statistic::ReservedRunes.into(),
        rune_counter: reserved_runes + 1,
      };
      self.statistics.insert(&statistic)?;

      Rune::reserved(reserved_runes.into())
    };

    // Nota bene: Because it would require constructing a block
    // with 2**16 + 1 transactions, there is no test that checks that
    // an eching in a transaction with an out-of-bounds index is
    // ignored.
    let Ok(index) = u16::try_from(index) else {
      return Ok(None);
    };

    Ok(Some(Etched {
      balance: if let Some(mint) = etching.mint {
        if mint.term == Some(0) {
          0
        } else {
          mint.limit.unwrap_or(runes::MAX_LIMIT)
        }
      } else {
        u128::MAX
      },
      divisibility: etching.divisibility,
      id: u128::from(self.height) << 16 | u128::from(index),
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

  fn claim(&self, id: u128) -> Result<Option<Claim>> {
    let Ok(key) = RuneId::try_from(id) else {
      return Ok(None);
    };

    let Some(entry) = self.id_to_entry.get(&key.store())? else {
      return Ok(None);
    };

    let entry = RuneEntry::load(entry.value());

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

  fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<u128, u128>> {
    // map of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<u128, u128> = HashMap::new();

    // increment unallocated runes with the runes in tx inputs
    for input in &tx.input {
      if let Some(guard) = self
        .outpoint_to_balances
        .remove(&input.previous_output.store())?
      {
        let buffer = guard.value();
        let mut i = 0;
        while i < buffer.len() {
          let (id, len) = varint::decode(&buffer[i..]);
          i += len;
          let (balance, len) = varint::decode(&buffer[i..]);
          i += len;
          *unallocated.entry(id).or_default() += balance;
        }
      }
    }

    Ok(unallocated)
  }
}
