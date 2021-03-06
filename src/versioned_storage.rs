use std::convert::TryInto;

use super::VersionedValue;

#[derive(Debug)]
pub struct VersionedStorage {
    pub(crate) db: sled::Db,
}

impl VersionedStorage {
    pub(crate) fn get(&self, key: &[u8]) -> Option<VersionedValue> {
        let raw_value = self.db.get(key).expect("db io issue")?;

        let ballot = u64::from_le_bytes(raw_value[..8].try_into().unwrap());

        Some(VersionedValue {
            ballot,
            value: if raw_value[8..].is_empty() {
                None
            } else {
                Some(raw_value[8..].to_vec())
            },
        })
    }

    pub(crate) fn update_if_newer(
        &self,
        key: &[u8],
        proposal: VersionedValue,
    ) -> Result<(), VersionedValue> {
        // we use a sled transaction to push all concurrency concerns into the db,
        // so we can be as massively concurrent as we desire in the rest of the
        // server code
        let ret = {
            let raw_value_opt = self.db.get(&key).expect("db io issue");

            let (current_ballot, current_value) =
                if let Some(ref raw_value) = raw_value_opt {
                    let current_ballot =
                        u64::from_le_bytes(raw_value[..8].try_into().unwrap());

                    let current_value = if raw_value[8..].is_empty() {
                        None
                    } else {
                        Some(raw_value[8..].to_vec())
                    };
                    (current_ballot, current_value)
                } else {
                    (0, None)
                };

            if proposal.ballot > current_ballot {
                let mut serialized: Vec<u8> =
                    proposal.ballot.to_le_bytes().to_vec();

                if let Some(value) = &proposal.value {
                    serialized.extend_from_slice(value);
                }

                self.db.insert(&*key, serialized).expect("db io issue");

                Ok(())
            } else {
                Err(VersionedValue {
                    ballot: current_ballot,
                    value: current_value,
                })
            }
        };

        // fsync our result before communicating anything to the client
        self.db.flush().expect("db io issue");

        ret
    }
}
