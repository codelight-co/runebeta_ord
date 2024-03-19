use diesel::{PgConnection, RunQueryDsl};

use crate::{
  runebeta::models::{NewSequenceNumberRuneId, SequenceNumberRuneId},
  schema::sequence_number_runeids::dsl::*,
};

pub struct SequenceNumberRuneIdTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> SequenceNumberRuneIdTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  // pub fn insert(
  //   &self,
  //   height_value: &i64,
  //   sequence_number_value: &i64,
  // ) -> Result<HeightSequenceNumber, diesel::result::Error> {
  //   match height_sequence_numbers
  //     .filter(height.eq(height_value))
  //     .returning(HeightSequenceNumber::as_returning())
  //     .first(self.connection)
  //     .optional()
  //   {
  //     Some(record) => diesel::update(height_sequence_numbers.find(record.id))
  //       .set(sequence_number_value.eq(sequence_number_value))
  //       .returning(HeightSequenceNumber::as_returning())
  //       .get_result(self.connection),
  //     None => {
  //       let payload = NewHeightSequenceNumber {
  //         height: height_value,
  //         sequence_number: sequence_number_value,
  //       };
  //       diesel::insert_into(height_sequence_numbers::table)
  //         .values(payload)
  //         .returning(HeightSequenceNumber::as_returning())
  //         .get_result(self.connection)
  //       //.expect("Error saving satpoint")
  //     }
  //   }
  // }
}
