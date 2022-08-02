use rocksdb::{MergeOperands, Options, DB};
use tracing::debug;

pub struct CounterDb {
    db_client: DB,
}

impl CounterDb {
    pub fn new() -> Self {
        let db_path = std::env::var("DB_PATH").unwrap();
        let mut opts = Options::default();

        opts.create_if_missing(true);
        opts.set_merge_operator_associative("inc operator", increment_merge);
        let db_client = DB::open(&opts, db_path).unwrap();

        Self { db_client }
    }

    pub fn inc_key(&self, key: &str, inc_value: u64) {
        // TODO: Actual error handling
        self.db_client.merge(key, inc_value.to_be_bytes()).unwrap();
    }

    pub fn get_key(&self, key: &str) -> u64 {
        // TODO: Actual error handling
        debug!("Getting key: {}", key);
        let val = self.db_client.get(key).unwrap().unwrap();
        u64::from_be_bytes(val.try_into().unwrap())
    }
}

fn increment_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut val = if let Some(old_val) = existing_val {
        // TODO: Actual error handling
        u64::from_be_bytes(old_val.try_into().unwrap())
    } else {
        0
    };

    for op in operands {
        let inc_amount = u64::from_be_bytes(op.try_into().unwrap());

        val += inc_amount;
    }

    let result = val.to_be_bytes().into_iter().collect();
    Some(result)
}
