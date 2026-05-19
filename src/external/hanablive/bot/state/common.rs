use std::collections::HashMap;

use crate::external::hanablive::dto::table::Table;

#[derive(Default, Debug)]
pub struct CommonState {
    pub tables: HashMap<usize, Table>,
}
