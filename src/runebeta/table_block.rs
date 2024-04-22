use diesel::{
  associations::HasTable, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
  RunQueryDsl, SelectableHelper,
};

use super::{
  models::{Block, NewBlock},
  InsertRecords,
};
use crate::schema::blocks::dsl::*;
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct BlockTable {}

impl<'conn> BlockTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn get_latest_block_height(
    &self,
    connection: &mut PgConnection,
  ) -> Result<i64, diesel::result::Error> {
    let block = blocks
      .select(Block::as_select())
      .order_by(block_height.desc())
      .first(connection)
      .optional()?; // This allows for returning an Option<Post>, otherwise it will throw an error
    Ok(block.map(|res| res.block_height).unwrap_or_default())
  }
  pub fn insert(
    &self,
    block: &NewBlock,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(blocks::table())
      .values(block)
      .on_conflict(block_height)
      .do_update()
      .set(block)
      .returning(Block::as_returning())
      .execute(connection)
  }
}

impl InsertRecords for BlockTable {
  const TABLE_NAME: &'static str = "blocks";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewBlock;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(blocks::table())
      .values(records)
      .on_conflict_do_nothing()
      .returning(Block::as_returning())
      .execute(connection)
  }
  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(blocks::table())
      .values(record)
      .on_conflict_do_nothing()
      .returning(Block::as_returning())
      .execute(connection)
  }
}
