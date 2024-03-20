use diesel::{
  deserialize::{FromSql, FromSqlRow},
  dsl::IsNull,
  pg::{Pg, PgValue},
  prelude::*,
  serialize::{Output, ToSql},
  sql_types::{Binary, Jsonb, Text},
  AsExpression,
};
//https://stackoverflow.com/questions/77629993/error-extending-diesel-with-wrapper-type-for-u128
#[derive(
  FromSqlRow,
  AsExpression,
  serde::Serialize,
  serde::Deserialize,
  Debug,
  PartialEq,
  Eq,
  PartialOrd,
  Default,
)]
#[diesel(sql_type = Text)]
pub struct U128(pub u128);

impl From<u128> for U128 {
  fn from(v: u128) -> U128 {
    U128(v)
  }
}

impl From<U128> for u128 {
  fn from(v: U128) -> u128 {
    v.0
  }
}

impl ToSql<Text, Pg> for U128 {
  fn to_sql<'b>(&self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    write!(out, "{}", self.0.to_string())?;
    Ok(IsNull::No)
  }
}
impl ToSql<Binary, Pg> for U128 {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    write!(out, "{}", self.0.to_ne_bytes())?;
    Ok(IsNull::No)
  }
}

impl FromSql<Text, Pg> for U128 {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let s = String::from_utf8_lossy(bytes.as_bytes());
    Ok(U128(s.parse()?))
  }
}

impl FromSql<Binary, Pg> for U128 {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = u128::from_ne_bytes(*(bytes.as_bytes()));
    Ok(U128(value))
  }
}
#[derive(FromSqlRow, AsExpression, serde::Serialize, serde::Deserialize, Debug, Default)]
#[diesel(sql_type = Jsonb)]
pub struct MintEntry {}

impl FromSql<Jsonb, Pg> for MintEntry {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl ToSql<Jsonb, Pg> for MintEntry {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
  }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutpointRuneBalance {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i32,
  pub rune_block: i32,
  pub rune_tx: i16,
  #[diesel(serialize_as = U128, deserialize_as = U128)]
  pub balance_value: u128,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
pub struct NewOutpointRuneBalance<'a> {
  pub tx_hash: &'a str,
  pub vout: i32,
  pub rune_block: i32,
  pub rune_tx: i16,
  pub balance_value: &'a U128,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_satranges)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutpointSatRange {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i16,
  pub range: Vec<u8>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::outpoint_satranges)]
pub struct NewOutpointSatRange<'a> {
  pub tx_hash: &'a str,
  pub vout: i16,
  pub range: &'a Vec<u8>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_values)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutPointValue {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i16,
  pub value: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::outpoint_values)]
pub struct NewOutPointValue<'a> {
  pub tx_hash: &'a str,
  pub vout: i16,
  pub value: i64,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RuneEntries {
  pub id: i64,
  pub rune_height: i32,
  pub rune_index: i16,
  #[diesel(deserialize_as = U128)]
  pub burned: u128,
  pub divisibility: i16,
  pub etching: String,
  pub mint: Option<MintEntry>,
  pub mints: i64,
  pub number: i64,
  pub rune: U128,
  pub spacers: i32,
  pub supply: U128,
  pub symbol: Option<String>,
  pub timestamp: i32,
}

#[derive(Insertable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRuneEntries<'a> {
  pub rune_height: i32,
  pub rune_index: i16,
  #[diesel(serialize_as = U128)]
  pub burned: U128,
  pub divisibility: i16,
  pub etching: &'a str,
  pub mint: Option<MintEntry>,
  pub mints: i64,
  pub number: i64,
  pub rune: U128,
  pub spacers: i32,
  pub supply: U128,
  pub symbol: Option<&'a str>,
  pub timestamp: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Rune {
  pub id: i64,
  pub rune: U128,
  pub tx_height: i64,
  pub rune_index: i16,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRune {
  pub rune: U128,
  pub tx_height: i64,
  pub rune_index: i16,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::sequence_number_runeids)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SequenceNumberRuneId {
  pub id: i64,
  pub sequence_number: i32,
  pub tx_height: i64,
  pub rune_index: i16,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexingStatistic {
  pub id: i32,
  pub schema: i32,
  pub blessed_inscriptions: i32,
  pub commits: i32,
  pub cursed_inscriptions: i32,
  pub index_runes: bool,
  pub index_sats: bool,
  pub lost_sats: i32,
  pub outputs_traversed: i32,
  pub reserved_runes: i64,
  pub runes: i64,
  pub satranges: i64,
  pub unbound_inscriptions: i32,
  pub index_transactions: bool,
  pub index_spent_sats: bool,
  pub initial_sync_time: i64,
}

#[derive(Default, Insertable)]
#[diesel(table_name = crate::schema::statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIndexingStatistic {
  pub schema: i32,
  pub blessed_inscriptions: i32,
  pub commits: i32,
  pub cursed_inscriptions: i32,
  pub index_runes: bool,
  pub index_sats: bool,
  pub lost_sats: i32,
  pub outputs_traversed: i32,
  pub reserved_runes: i64,
  pub runes: i64,
  pub satranges: i64,
  pub unbound_inscriptions: i32,
  pub index_transactions: bool,
  pub index_spent_sats: bool,
  pub initial_sync_time: i64,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxidRune {
  pub id: i64,
  pub tx_hash: String,
  pub rune: U128,
}

///Models for create

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sequence_number_runeids)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSequenceNumberRuneId {
  pub sequence_number: i32,
  pub tx_height: i64,
  pub rune_index: i16,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTxidRune<'a> {
  pub tx_hash: &'a str,
  pub rune: U128,
}

//ContentTypeCounts
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::content_type_counts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentTypeCount {
  pub id: i32,
  pub content_type: Option<String>,
  pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::content_type_counts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContentTypeCount {
  pub content_type: Option<String>,
  pub count: i64,
}

//Inscription
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::inscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Inscriptions {
  pub id: i64,
  pub home: Option<i32>,
  pub sequence_number: i32,
  pub head: U128,
  pub tail: U128,
  pub inscription_index: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::inscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewInscriptions {
  pub home: Option<i32>,
  pub sequence_number: i32,
  pub head: U128,
  pub tail: U128,
  pub inscription_index: i32,
}

//InscriptionEntry
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::inscription_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InscriptionEntry {
  pub id: i64,
  pub charms: i16,
  pub fee: i64,
  pub height: i32,
  pub tx_hash: String,
  pub inscription_index: i32,
  pub inscription_number: i32,
  pub parent: Option<i32>,
  pub sat: Option<i64>,
  pub sequence_number: i32,
  pub timestamp: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::inscription_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewInscriptionEntry {
  pub charms: i16,
  pub fee: i64,
  pub height: i32,
  pub tx_hash: String,
  pub inscription_index: i32,
  pub inscription_number: i32,
  pub parent: Option<i32>,
  pub sat: Option<i64>,
  pub sequence_number: i32,
  pub timestamp: i32,
}

//Satpoint
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::satpoints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SatpointEntity {
  pub id: i64,
  pub sequence_number: i32,
  pub tx_hash: String,
  pub vout: i32,
  pub sat_offset: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::satpoints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSatpointEntity {
  pub sequence_number: i32,
  pub tx_hash: String,
  pub vout: i32,
  pub sat_offset: i64,
}

//BlockHeader
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::block_headers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockHeader {
  pub id: i64,
  pub height: i64,
  pub version: i32,
  pub previous_block_hash: String,
  pub merkle_root: String,
  pub time: i32,
  pub bits: i32,
  pub nonce: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::block_headers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewBlockHeader {
  pub height: i64,
  pub version: i32,
  pub previous_block_hash: String,
  pub merkle_root: String,
  pub time: i32,
  pub bits: i32,
  pub nonce: i32,
}

//Transaction
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
  pub id: i64,
  pub version: i32,
  pub lock_time: i32,
  pub tx_hash: String,
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
  pub version: i32,
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
  pub sequence_number: i32,
  pub witness: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionIn {
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i32,
  pub script_sig: String,
  pub sequence_number: i32,
  pub witness: String,
}

//TransactionOut
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionOut {
  pub id: i64,
  pub tx_hash: String,
  pub value: i64,
  pub script_pubkey: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionOut {
  pub tx_hash: String,
  pub value: i64,
  pub script_pubkey: String,
}

//HeightSequenceNumber
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::height_sequence_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HeightSequenceNumber {
  pub id: i64,
  pub height: i32,
  pub sequence_number: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::height_sequence_numbers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHeightSequenceNumber {
  pub height: i32,
  pub sequence_number: i32,
}

//BlockTimestamp
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::indexing_block_timestamps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexingBlockTimestamp {
  pub id: i64,
  pub block_height: i32,
  pub timestamps: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::indexing_block_timestamps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIndexingBlockTimestamp {
  pub block_height: i32,
  pub timestamps: i64,
}
