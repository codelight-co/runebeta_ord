use std::{collections::BTreeMap, sync::Arc};

use axum::{extract::Path, response::IntoResponse, routing::get, Extension, Json, Router};
use bitcoin::{Address, OutPoint, Txid};
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
    Extension(settings): Extension<Arc<Settings>>,
    Extension(index): Extension<Arc<Index>>,
    Path(address): Path<String>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult {
    task::block_in_place(|| {
      //Get all transaction
      let balances = index.get_rune_balance_map()?;
      let mut outpoints = BTreeMap::<Txid, u32>::new();
      balances.iter().for_each(|(_, balances)| {
        balances.iter().for_each(|(outpoint, _)| {
          outpoints.insert(outpoint.txid.clone(), outpoint.vout);
        })
      });
      let mut tx_outs = index.get_transaction_outs(outpoints)?;
      //Filter by address
      tx_outs = tx_outs
        .into_iter()
        .filter(|(_, txout)| {
          if let Ok(addr) =
            Address::from_script(txout.script_pubkey.as_script(), settings.chain().network())
          {
            addr.to_string() == address.as_str()
          } else {
            false
          }
        })
        .collect();
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
