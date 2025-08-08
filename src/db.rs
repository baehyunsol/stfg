use crate::table::Table;
use crate::view::View;

/// Tables and views are always sorted by name.
pub struct DB {
    pub tables: Vec<Table>,
    pub views: Vec<View>,
}
