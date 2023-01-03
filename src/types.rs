use paperclip::v2::models::{DataType, DataTypeFormat, DefaultSchemaRaw};
use paperclip::v2::schema::Apiv2Schema;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct RecursiveTypeWrapper<T>(pub T);

impl<T> Apiv2Schema for RecursiveTypeWrapper<T> {
    fn required() -> bool {
        true
    }

    fn raw_schema() -> DefaultSchemaRaw {
        DefaultSchemaRaw {
            data_type: Some(DataType::Array),
            format: Some(DataTypeFormat::Other),
            ..Default::default()
        }
    }
}

pub type MeetID = i32;
pub type SpecializationID = i32;
pub type ServiceID = i32;
pub type PeopleID = i32;
pub type AccountID = i64;
pub type RangeTime = i64;
pub type EducationID = i32;
pub type AppStateID = i64;
