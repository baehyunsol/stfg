use crate::error::Error;
use crate::record::read_records;
use crate::to_git::get_db_schema_from_raw_sql;
use ragit_fs::{
    basename,
    exists,
    is_dir,
    join,
    read_dir,
    read_string,
    remove_file,
};
use rusqlite::{Connection, params_from_iter};

pub fn from_git(
    db_path: &str,
    data_dir: &str,
) -> Result<(), Error> {
    if exists(db_path) {
        remove_file(db_path)?;
    }

    let mut conn = Connection::open(db_path)?;

    for table_dir in read_dir(data_dir, false)?.iter() {
        if !is_dir(table_dir) && basename(&table_dir)? == ".empty" {
            continue;
        }

        let table_sql = read_string(&join(table_dir, "table.sql")?)?;
        conn.execute(&table_sql, [])?;
        let tx = conn.transaction()?;

        let table_schema = get_db_schema_from_raw_sql(&table_sql)?;
        let table_schema = match table_schema.len() {
            1 => table_schema[0].clone(),
            n => {
                return Err(Error::CorruptedDataFile(format!("expected exactly 1 `CREATE TABLE` statement from `table.sql`, but got {n}")));
            },
        };
        let insert_stmt = table_schema.insert_stmt();
        let mut insert_stmt = tx.prepare(&insert_stmt)?;

        for data_file in read_dir(&table_dir, false)?.iter() {
            let data_file_name = basename(&data_file)?;

            if data_file_name.len() != 3 {
                continue;
            }

            let records = read_records(data_file)?;

            for record in records.iter() {
                insert_stmt.execute(params_from_iter(record.fields.iter().map(|(_, v)| v)))?;
            }
        }

        drop(insert_stmt);
        tx.commit()?;

        let index_sql = read_string(&join(table_dir, "index.sql")?)?;

        if !index_sql.trim().is_empty() {
            conn.execute_batch(&index_sql)?;
        }
    }

    Ok(())
}
