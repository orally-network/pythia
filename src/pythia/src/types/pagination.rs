use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, CandidType)]
pub struct Pagination {
    pub from: usize,
    pub size: usize,
}

impl Pagination {
    pub fn paginate<T: Clone>(&self, data: Vec<T>) -> Vec<T> {
        let total_items = data.len();
        let start_index = std::cmp::min(self.from, total_items);
        let end_index = std::cmp::min(start_index + self.size, total_items);

        data[start_index..end_index].to_vec()
    }
}
