use diesel::{
  associations::HasTable, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{NewRune, Rune, U128},
  schema::runes::dsl::*,
  subcommand::epochs::run,
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
  pub fn get(&mut self, rune_id: u128) -> Result<Option<Rune>, diesel::result::Error> {
    let key = U128(rune_id);
    let res = runes
      .filter(rune.eq(&key))
      .select(Rune::as_select())
      .first(self.connection)
      .ok(); // This allows for returning an Option<Post>, otherwise it will throw an error
    Ok(res)
  }
}
