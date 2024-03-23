use std::{collections::BTreeMap, sync::Arc, time::Instant};

use axum::{extract::Path, response::IntoResponse, routing::get, Extension, Json, Router};
use bitcoin::{OutPoint, Txid};
use tokio::task;

use crate::{
  settings::Settings,
  templates::{PageContent, RuneBalancesHtml},
  Index, Rune,
};

use super::{accept_json::AcceptJson, error::ServerResult, server::ServerConfig};
pub struct RunebetaServer {}
impl RunebetaServer {
  pub fn create_router() -> Router<Arc<ServerConfig>> {
    Router::new().route("/runes/balances/:address", get(Self::runes_balances))
  }
  async fn runes_balances(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(_settings): Extension<Arc<Settings>>,
    Extension(index): Extension<Arc<Index>>,
    Path(address): Path<String>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult {
    task::block_in_place(|| {
      //Get all transaction
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
}
