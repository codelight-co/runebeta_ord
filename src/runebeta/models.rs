use crate::Terms;
use bigdecimal::BigDecimal;
use diesel::{
  deserialize::{FromSql, FromSqlRow},
  pg::Pg,
  prelude::*,
  serialize::{IsNull, Output, ToSql},
  sql_types::Jsonb,
  AsExpression,
};
use ordinals::{Cenotaph, Edict, Etching, Rune, RuneId, Runestone};
use std::io::Write;
// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
// https://vasilakisfil.social/blog/2020/05/09/rust-diesel-jsonb/

#[derive(
  Clone,
  FromSqlRow,
  AsExpression,
  serde::Serialize,
  serde::Deserialize,
  Debug,
  Default,
  PartialEq,
  Eq,
  PartialOrd,
)]
#[diesel(sql_type = Jsonb)]
pub struct MintEntryType {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub amount: Option<BigDecimal>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cap: Option<BigDecimal>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height1: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height2: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset1: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset2: Option<i64>,
}

impl ToSql<Jsonb, Pg> for MintEntryType {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    // <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    out.write_all(&[1])?;
    serde_json::to_writer(out, &value)
      .map(|_| IsNull::No)
      .map_err(Into::into)
  }
}
impl FromSql<Jsonb, Pg> for MintEntryType {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl From<&Terms> for MintEntryType {
  fn from(value: &Terms) -> Self {
    let (height1, height2) = value.height.clone();
    let (offset1, offset2) = value.offset.clone();
    MintEntryType {
      amount: value.amount.map(|v| BigDecimal::from(v)),
      cap: value.cap.map(|v| BigDecimal::from(v)),
      height1: height1.map(|v| v as i64),
      height2: height2.map(|v| v as i64),
      offset1: offset1.map(|v| v as i64),
      offset2: offset2.map(|v| v as i64),
    }
  }
}

#[derive(Clone, FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[diesel(sql_type = Jsonb)]
pub struct RunestoneValue {
  pub edicts: Vec<Edict>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub etching: Option<Etching>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mint: Option<RuneId>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pointer: Option<u32>,
}
impl ToSql<Jsonb, Pg> for RunestoneValue {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    // <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    out.write_all(&[1])?;
    serde_json::to_writer(out, &value)
      .map(|_| IsNull::No)
      .map_err(Into::into)
  }
}
impl FromSql<Jsonb, Pg> for RunestoneValue {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl From<&Runestone> for RunestoneValue {
  fn from(value: &Runestone) -> Self {
    let Runestone {
      edicts,
      etching,
      mint,
      pointer,
    } = value;
    RunestoneValue {
      edicts: edicts.clone(),
      etching: etching.clone(),
      mint: mint.clone(),
      pointer: pointer.clone(),
    }
  }
}

#[derive(Clone, FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[diesel(sql_type = Jsonb)]
pub struct CenotaphValue {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub etching: Option<Rune>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mint: Option<RuneId>,
  pub flaws: u32,
}
impl ToSql<Jsonb, Pg> for CenotaphValue {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    // <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    out.write_all(&[1])?;
    serde_json::to_writer(out, &value)
      .map(|_| IsNull::No)
      .map_err(Into::into)
  }
}
impl FromSql<Jsonb, Pg> for CenotaphValue {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl From<&Cenotaph> for CenotaphValue {
  fn from(value: &Cenotaph) -> Self {
    let Cenotaph {
      etching,
      mint,
      flaws,
    } = value;
    CenotaphValue {
      etching: etching.clone(),
      mint: mint.clone(),
      flaws: flaws.clone(),
    }
  }
}
//Block
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Block {
  pub id: i64,
  pub block_time: i64,
  pub block_height: i64,
  pub previous_hash: String,
  pub block_hash: String,
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewBlock {
  pub block_time: i64,
  pub block_height: i64,
  pub previous_hash: String,
  pub block_hash: String,
}

//Transaction
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
  pub id: i64,
  pub block_height: i64,
  pub version: i32,
  pub lock_time: i32,
  pub tx_hash: String,
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
  pub version: i32,
  pub block_height: i64,
  pub lock_time: i32,
  pub tx_hash: String,
}

//TransactionIn
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionIn {
  pub id: i64,
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i32,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionIn {
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i32,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

//TransactionOut
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionOut {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i64,
  pub value: i64,
  pub asm: String,
  pub dust_value: i64,
  pub address: Option<String>,
  pub script_pubkey: String,
  pub spent: bool,
  pub runestone: RunestoneValue,
  pub cenotaph: CenotaphValue,
  pub edicts: i32,
  pub etching: bool,
  pub mint: bool,
  pub burn: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionOut {
  pub tx_hash: String,
  pub vout: i64,
  pub value: i64,
  pub asm: String,
  pub dust_value: i64,
  pub address: Option<String>,
  pub script_pubkey: String,
  pub spent: bool,
  pub runestone: RunestoneValue,
  pub cenotaph: CenotaphValue,
  pub edicts: i32,
  pub etching: bool,
  pub mint: bool,
  pub burn: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxRuneEntry {
  pub id: i64,
  pub tx_hash: String,
  // pub rune_height: i32,
  // pub rune_index: i16,
  pub rune_id: String,
  pub burned: BigDecimal,
  pub divisibility: i16,
  pub etching: String,
  pub mint_entry: MintEntryType,
  pub mints: i64,
  pub number: i64,
  pub rune: BigDecimal,
  pub spacers: i32,
  pub premine: i64,
  pub spaced_rune: String,
  pub supply: BigDecimal,
  pub symbol: Option<String>,
  pub timestamp: i32,
}

#[derive(Insertable, PartialEq, Clone, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTxRuneEntry {
  pub tx_hash: String,
  // pub rune_height: i32,
  // pub rune_index: i16,
  pub rune_id: String,
  pub burned: BigDecimal,
  pub divisibility: i16,
  pub etching: String,
  pub mint_entry: MintEntryType,
  pub mints: i64,
  pub number: i64, //Block
  pub rune: BigDecimal,
  pub spacers: i32,
  pub premine: i64,
  pub spaced_rune: String,
  pub supply: BigDecimal,
  pub symbol: Option<String>,
  pub timestamp: i32,
}

//TransactionRune
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRune {
  pub id: i64,
  pub tx_hash: String,
  pub rune_id: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRune {
  pub tx_hash: String,
  pub rune_id: String,
}

//TransactionRuneIdAddress
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_rune_addresss)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRuneAddress {
  pub id: i64,
  pub tx_hash: String,
  pub rune_id: String,
  pub address: String,
  pub spent: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::txid_rune_addresss)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRuneAddress {
  pub tx_hash: String,
  pub rune_id: String,
  pub address: String,
  pub spent: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutpointRuneBalance {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i32,
  pub rune_id: String,
  pub balance_value: BigDecimal,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
pub struct NewOutpointRuneBalance {
  pub tx_hash: String,
  pub vout: i32,
  pub rune_id: String,
  pub balance_value: BigDecimal,
}
