use axum::{extract::Path, response::IntoResponse, routing::get, Extension, Json, Router};
use bitcoin::{Address, ScriptBuf, TxOut};
use bitcoin::{OutPoint, Txid};
use std::{collections::BTreeMap, sync::Arc, time::Instant};
use tokio::task;

use crate::{
  settings::Settings,
  subcommand::server::error::OptionExt,
  templates::{OutputHtml, PageContent, RuneBalancesHtml},
  Index, Rune,
};

use super::*;
pub struct RunebetaServer {}
impl RunebetaServer {
  pub fn create_router() -> Router<Arc<ServerConfig>> {
    Router::new()
      .route("/runes/balances/:address", get(Self::runes_balances))
      .route("/output/:output", get(Self::output))
  }
  async fn runes_balances(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(_settings): Extension<Arc<Settings>>,
    Extension(index): Extension<Arc<Index>>,
    Path(address): Path<String>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult {
    task::block_in_place(|| {
      log::info!("Start get runes balances by address");
      let mut now = Instant::now();
      let balances = index.get_rune_balance_map()?;
      let mut outpoints = BTreeMap::<Txid, u32>::new();
      balances.iter().for_each(|(_, balances)| {
        balances.iter().for_each(|(outpoint, _)| {
          outpoints.insert(outpoint.txid.clone(), outpoint.vout);
        })
      });
      log::info!(
        "get_rune_balance finished in {}ms",
        now.elapsed().as_millis()
      );
      now = Instant::now();
      let tx_outs = index.get_transaction_outs(outpoints, address.as_str())?;
      log::info!(
        "filter transaction outs finished in {}ms",
        now.elapsed().as_millis()
      );
      Ok(if accept_json {
        Json(
          balances
            .into_iter()
            .map(|(rune, balances)| {
              (
                rune,
                balances
                  .into_iter()
                  .filter(|(outpoint, _)| tx_outs.contains_key(outpoint))
                  .map(|(outpoint, pile)| (outpoint, pile.amount))
                  .collect(),
              )
            })
            .collect::<BTreeMap<Rune, BTreeMap<OutPoint, u128>>>(),
        )
        .into_response()
      } else {
        RuneBalancesHtml { balances }
          .page(server_config)
          .into_response()
      })
    })
  }

  async fn output(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(settings): Extension<Arc<Settings>>,
    Extension(index): Extension<Arc<Index>>,
    Path(outpoint): Path<OutPoint>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult {
    task::block_in_place(|| {
      let sat_ranges = index.list(outpoint)?;

      let indexed;

      let output = if outpoint == OutPoint::null() || outpoint == unbound_outpoint() {
        let mut value = 0;

        if let Some(ranges) = &sat_ranges {
          for (start, end) in ranges {
            value += end - start;
          }
        }

        indexed = true;

        TxOut {
          value,
          script_pubkey: ScriptBuf::new(),
        }
      } else {
        indexed = index.contains_output(&outpoint)?;

        index
          .get_transaction(outpoint.txid)?
          .ok_or_not_found(|| format!("output {outpoint}"))?
          .output
          .into_iter()
          .nth(outpoint.vout as usize)
          .ok_or_not_found(|| format!("output {outpoint}"))?
      };
      if let Ok(address) =
        Address::from_script(output.script_pubkey.as_script(), settings.chain().network())
      {
        log::info!(
          "Address {}, address.to_string(): {}",
          &address,
          address.to_string()
        );
      } else {
        log::info!("Cannot parse address from txout {:?}", &output);
      }
      let inscriptions = index.get_inscriptions_on_output(outpoint)?;

      let runes = index.get_rune_balances_for_outpoint(outpoint)?;

      let spent = index.is_output_spent(outpoint)?;

      Ok(if accept_json {
        Json(api::Output::new(
          server_config.chain,
          inscriptions,
          outpoint,
          output,
          indexed,
          runes,
          sat_ranges,
          spent,
        ))
        .into_response()
      } else {
        OutputHtml {
          chain: server_config.chain,
          inscriptions,
          outpoint,
          output,
          runes,
          sat_ranges,
          spent,
        }
        .page(server_config)
        .into_response()
      })
    })
  }
}
