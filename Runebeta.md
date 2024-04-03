# Runebeta code
1. runebeta folder
2. migrations folder
3. diesel.toml
4. run.sh
5. docker-compose.xml
7. Cargo.toml
```
# runebeta
diesel = { version = "2.1", features = ["postgres", "uuid", "serde_json", "numeric"] }
bigdecimal =  { version = "0.4.3" }
deadpool-diesel = { version = "0.4", features = ["postgres"] }
dotenvy = "0.15"
# End runebeta

```
8. src/lib.rs
Add runebeta module
mod runebeta;
pub use runebeta::*,
3. src/index/updater.rs

```
    // If value_receiver still has values something went wrong with the last block
    // Could be an assert, shouldn't recover from this and commit the last block
    let Err(TryRecvError::Empty) = value_receiver.try_recv() else {
      return Err(anyhow!("Previous block did not consume all input values"));
    };

    let mut outpoint_to_value = wtx.open_table(OUTPOINT_TO_VALUE)?;

    let index_inscriptions = self.height >= self.index.first_inscription_height
      && self.index.settings.index_inscriptions();
    
    
    //Start add runebeta extension here
    let extension = IndexExtension::new(
      self.index.settings.chain(),
      self.height as i64,
      block.header.clone(),
    );
    if block.txdata.len() > 0 && index_inscriptions {
      //Index block with data only
      let _res = extension.index_block(&block.txdata);
    }

    // End of runebeta extension
    
```

```
    let mut rune_updater = RuneUpdater {
        ...
        extension: Some(extension), // Add externsion here
      };
```

4. src/index/updater/rune_updater.rs

```
  // Sort balances by id so tests can assert balances in a fixed order
  balances.sort();

  if let Some(extension) = &self.extension {
      let _res = extension.index_outpoint_balances(
        &txid,
        vout as i32,
        &balances
          .iter()
          .map(|(rune_id, balance)| (rune_id.clone(), BigDecimal::from(balance.0)))
          .collect(),
      );
    }
```

Line 286
```
    /*
     * Taivv April 03, index data to postgres
     */
    if let Some(extension) = &self.extension {
      let _ = extension.index_transaction_rune_entry(&txid, &id, &entry);
    }

    self.id_to_entry.insert(id.store(), entry.store())?;
```

5. Run with docker

```
docker-compose --env-file .env.testnet up -d
```