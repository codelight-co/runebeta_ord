use diesel::{
  associations::HasTable, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{NewRuneEntries, RuneEntries},
  schema::{rune_entries::dsl::*, runes::rune},
  RuneId,
};

pub struct RuneEntryTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> RuneEntryTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(&self, payload: &NewRuneEntries) -> Result<RuneEntries, diesel::result::Error> {
    diesel::insert_into(rune_entries::table())
      .values(payload)
      .returning(RuneEntries::as_returning())
      .get_result(self.connection)
    //.expect("Error saving satpoint")
  }
  pub fn update(&mut self, payload: &RuneEntries) -> Result<RuneEntries, diesel::result::Error> {
    diesel::update(rune_entries.find(payload.id))
      .set((
        rune_height.eq(payload.rune_height),
        rune_index.eq(payload.rune_index),
        burned.eq(payload.burned),
        divisibility.eq(payload.divisibility),
        etching.eq(payload.etching),
        mint.eq(payload.mint),
        number.eq(payload.number),
        spacers.eq(payload.spacers),
        supply.eq(payload.supply),
      ))
      .returning(RuneEntries::as_returning())
      .get_result(self.connection)
    //.expect("Error saving satpoint")
  }
  pub fn get(&mut self, rune_id: &RuneId) -> Result<Option<RuneEntries>, diesel::result::Error> {
    let height = rune_id.block as i32;
    let index = rune_id.tx as i16;
    rune_entries
      .filter(rune_height.eq(&height))
      .filter(rune_index.eq(&index))
      .limit(1)
      .select(RuneEntries::as_select())
      .load::<RuneEntries>(self.connection)
      .map(|vec| vec.pop())
    //.expect("Error saving satpoint")
  }
}
