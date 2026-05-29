use chrono::NaiveDate;

use crate::domain::entities::EncryptedNoteRecord;
use crate::domain::errors::DomainResult;

pub trait NoteRepository {
    fn create(&self, record: &EncryptedNoteRecord) -> DomainResult<()>;
    fn update(&self, record: &EncryptedNoteRecord) -> DomainResult<()>;
    fn delete(&self, id: &str) -> DomainResult<()>;
    fn get_by_id(&self, id: &str) -> DomainResult<Option<EncryptedNoteRecord>>;
    fn get_by_date(&self, date: NaiveDate) -> DomainResult<Option<EncryptedNoteRecord>>;
    fn list_by_date_range(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> DomainResult<Vec<EncryptedNoteRecord>>;
    fn list_favorites(&self) -> DomainResult<Vec<EncryptedNoteRecord>>;
    fn list_future(&self, after: NaiveDate) -> DomainResult<Vec<EncryptedNoteRecord>>;
    fn list_for_month(&self, year: i32, month: u32) -> DomainResult<Vec<EncryptedNoteRecord>>;
    fn list_all(&self) -> DomainResult<Vec<EncryptedNoteRecord>>;
}
