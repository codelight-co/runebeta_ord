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
impl FromSql<Text, Pg> for U128 {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let s = String::from_utf8_lossy(bytes.as_bytes());
    Ok(U128(s.parse()?))
  }
}

// impl ToSql<Binary, Pg> for U128 {
//   fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
//     write!(out, "{}", self.0.to_ne_bytes())?;
//     Ok(IsNull::No)
//   }
// }

// impl FromSql<Binary, Pg> for U128 {
//   fn from_sql(
//     bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
//   ) -> diesel::deserialize::Result<Self> {
//     let value = u128::from_ne_bytes(*(bytes.as_bytes()));
//     Ok(U128(value))
//   }
// }

#[derive(
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
pub struct MintEntryType {}

impl ToSql<Jsonb, Pg> for MintEntryType {
  fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    todo!()
  }
}
impl FromSql<Jsonb, Pg> for MintEntryType {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    todo!()
  }
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

#[derive(Insertable, AsChangeset)]
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

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionOut {
  pub tx_hash: String,
  pub value: i64,
  pub script_pubkey: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxRuneEntry {
  pub id: i64,
  pub tx_hash: String,
  pub rune_height: i32,
  pub rune_index: i16,
  #[diesel(deserialize_as = U128)]
  pub burned: u128,
  pub divisibility: i16,
  pub etching: String,
  pub mint_entry: Option<MintEntryType>,
  pub mints: i64,
  pub number: i64,
  pub rune: U128,
  pub spacers: i32,
  pub supply: U128,
  pub symbol: Option<String>,
  pub timestamp: i32,
}

#[derive(Insertable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTxRuneEntry<'a> {
  pub tx_hash: String,
  pub rune_height: i32,
  pub rune_index: i16,
  #[diesel(serialize_as = U128)]
  pub burned: U128,
  pub divisibility: i16,
  pub etching: &'a str,
  pub mint_entry: Option<MintEntryType>,
  pub mints: i64,
  pub number: i64,
  pub rune: U128,
  pub spacers: i32,
  pub supply: U128,
  pub symbol: Option<&'a str>,
  pub timestamp: i32,
}
