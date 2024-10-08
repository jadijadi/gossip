use crate::error::Error;
use crate::storage::{RawDatabase, Storage};
use heed::{types::Bytes, DatabaseFlags};
use std::sync::Mutex;

// Kind:Pubkey:d-tag -> RelationshipByAddr2:Id
//   (has dups)

static RELATIONSHIPS_BY_ADDR2_DB_CREATE_LOCK: Mutex<()> = Mutex::new(());
static mut RELATIONSHIPS_BY_ADDR2_DB: Option<RawDatabase> = None;

impl Storage {
    pub(super) fn db_relationships_by_addr2(&self) -> Result<RawDatabase, Error> {
        unsafe {
            if let Some(db) = RELATIONSHIPS_BY_ADDR2_DB {
                Ok(db)
            } else {
                // Lock.  This drops when anything returns.
                let _lock = RELATIONSHIPS_BY_ADDR2_DB_CREATE_LOCK.lock();

                // In case of a race, check again
                if let Some(db) = RELATIONSHIPS_BY_ADDR2_DB {
                    return Ok(db);
                }

                // Create it. We know that nobody else is doing this and that
                // it cannot happen twice.
                let mut txn = self.env.write_txn()?;
                let db = self
                    .env
                    .database_options()
                    .types::<Bytes, Bytes>()
                    .flags(DatabaseFlags::DUP_SORT) // NOT FIXED, RelationshipByAddr2 serialized isn't.
                    .name("relationships_by_addr2")
                    .create(&mut txn)?;
                txn.commit()?;
                RELATIONSHIPS_BY_ADDR2_DB = Some(db);
                Ok(db)
            }
        }
    }
}
