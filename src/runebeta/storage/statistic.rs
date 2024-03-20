use diesel::{
  associations::HasTable, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{IndexingStatistic, NewIndexingStatistic},
  schema::statistics::{dsl::statistics, *},
};
pub struct StatisticTable<'conn> {
  pub connection: &'conn mut PgConnection,
}
impl<'conn> StatisticTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn get_or_create_indexing_statistic(
    &mut self,
  ) -> Result<IndexingStatistic, diesel::result::Error> {
    let indexing_statistic = match statistics
      .select(IndexingStatistic::as_select())
      .first(self.connection)
    {
      Ok(value) => Ok(value),
      Err(_) => self.create_indexing_statistic(&NewIndexingStatistic::default()),
    };
    indexing_statistic
  }

  pub fn set_initial_sync_time(
    &self,
    value: i64,
  ) -> Result<IndexingStatistic, diesel::result::Error> {
    match statistics
      .select(IndexingStatistic::as_select())
      .first(self.connection)
    {
      Ok(record) => diesel::update(statistics.find(record.id))
        .set(initial_sync_time.eq(&value))
        .get_result(self.connection),
      Err(_) => {
        let mut statistic = NewIndexingStatistic::default();
        statistic.initial_sync_time = value;
        self.create_indexing_statistic(&statistic)
      }
    }
  }
  pub fn set_runes(&mut self, value: i64) -> Result<IndexingStatistic, diesel::result::Error> {
    match statistics
      .select(IndexingStatistic::as_select())
      .first(self.connection)
    {
      Ok(record) => diesel::update(statistics.find(record.id))
        .set(runes.eq(value))
        .returning(IndexingStatistic::as_returning())
        .get_result(self.connection),
      Err(_) => {
        let mut statistic = NewIndexingStatistic::default();
        statistic.runes = value;
        self.create_indexing_statistic(&statistic)
      }
    }
  }
  pub fn get_reserved_runes(&mut self) -> Result<i64, diesel::result::Error> {
    statistics
      .select(reserved_runes)
      .limit(1)
      .get_result::<i64>(self.connection)
  }
  pub fn set_reserved_runes(
    &mut self,
    value: i64,
  ) -> Result<IndexingStatistic, diesel::result::Error> {
    match statistics
      .select(IndexingStatistic::as_select())
      .limit(1)
      .get_result(self.connection)
    {
      Ok(record) => diesel::update(statistics.find(record.id))
        .set(reserved_runes.eq(&value))
        .returning(IndexingStatistic::as_returning())
        .get_result(self.connection),
      Err(_) => {
        let mut statistic = NewIndexingStatistic::default();
        statistic.reserved_runes = value;
        self.create_indexing_statistic(&statistic)
      }
    }
  }
  pub fn create_indexing_statistic(
    &self,
    payload: &NewIndexingStatistic,
  ) -> Result<IndexingStatistic, diesel::result::Error> {
    diesel::insert_into(statistics::table())
      .values(payload)
      .returning(IndexingStatistic::as_returning())
      .get_result(self.connection)
  }
  pub fn update_indexing_statistic(
    &mut self,
    update_data: &IndexingStatistic,
  ) -> Result<IndexingStatistic, diesel::result::Error> {
    diesel::update(statistics.find(update_data.id))
      .set((
        schema.eq(update_data.schema),
        blessed_inscriptions.eq(update_data.blessed_inscriptions),
        commits.eq(update_data.commits),
        cursed_inscriptions.eq(update_data.cursed_inscriptions),
        cursed_inscriptions.eq(update_data.cursed_inscriptions),
        index_runes.eq(update_data.index_runes),
        index_sats.eq(update_data.index_sats),
        lost_sats.eq(update_data.lost_sats),
        outputs_traversed.eq(update_data.outputs_traversed),
        reserved_runes.eq(update_data.reserved_runes),
        runes.eq(update_data.runes),
        satranges.eq(update_data.satranges),
        unbound_inscriptions.eq(update_data.unbound_inscriptions),
        index_transactions.eq(update_data.index_transactions),
        index_spent_sats.eq(update_data.index_spent_sats),
        initial_sync_time.eq(update_data.initial_sync_time),
      ))
      .returning(IndexingStatistic::as_returning())
      .get_result(self.connection)
  }
}

// pub fn get_indexing_statistic(
//   connection: &mut PgConnection,
// ) -> Result<Option<IndexingStatistic>, anyhow::Error> {
//   let indexing_statistic = statistics
//     .select(IndexingStatistic::as_select())
//     .first(connection)
//     .optional(); // This allows for returning an Option<Post>, otherwise it will throw an error
//   Ok(indexing_statistic)
// }
// pub fn create_indexing_statistic(
//   payload: &NewIndexingStatistic,
//   connection: &mut PgConnection,
// ) -> IndexingStatistic {
//   diesel::insert_into(statistics::table)
//     .values(payload)
//     .returning(IndexingStatistic::as_returning())
//     .get_result(connection)
//     .expect("Error saving new post")
// }
