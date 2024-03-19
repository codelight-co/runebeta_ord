use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use crate::{
  runebeta::models::{NewRune, Rune},
  schema::runes::dsl::*,
};

pub struct RuneTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> RuneTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(&self, payload: &NewRune) -> Result<Rune, diesel::result::Error> {
    diesel::insert_into(runes::table())
      .values(payload)
      .returning(Rune::as_returning())
      .get_result(self.connection)
    //.expect("Error saving satpoint")
  }
}
