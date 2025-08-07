use crate::error::Error;
use crate::record::{
    Record,
    RecordId,
    read_records,
    write_records,
};
use crate::table::{Table, escape_path};
use crate::value::Value;
use ragit_fs::{
    WriteMode,
    create_dir_all,
    exists,
    join,
    remove_dir_all,
    write_string,
};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub fn to_git(
    db_path: &str,
    output_path: &str,
) -> Result<(), Error> {
    let db_schema = get_db_schema(db_path)?;
    dump_db(db_path, &db_schema, output_path)?;
    Ok(())
}

pub(crate) fn get_db_schema(db_path: &str) -> Result<Vec<Table>, Error> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    get_db_schema_worker(conn)
}

pub(crate) fn get_db_schema_from_raw_sql(sql: &str) -> Result<Vec<Table>, Error> {
    let conn = Connection::open_in_memory()?;
    conn.execute(sql, [])?;
    get_db_schema_worker(conn)
}

fn get_db_schema_worker(conn: Connection) -> Result<Vec<Table>, Error> {
    let mut tables_names: Vec<String> = vec![];
    let mut tables_by_name = HashMap::new();

    let mut tables_stmt = conn.prepare("SELECT * FROM pragma_table_list;")?;
    let mut table_stmt = conn.prepare("SELECT * FROM pragma_table_info(?1);")?;

    // TODO: if the name `sqlite_schema` is already used, I have to use `sqlite_master`.
    let mut sqls_stmt = conn.prepare("SELECT * FROM sqlite_schema;")?;

    let mut tables_q = tables_stmt.query([])?;

    while let Some(table_q) = tables_q.next()? {
        let table_name = table_q.get("name")?;
        tables_names.push(table_name);
    }

    for table_name in tables_names.iter() {
        let mut column_names: Vec<String> = vec![];
        let mut primary_key: Option<String> = None;
        let mut columns_q = table_stmt.query([table_name])?;

        while let Some(column_q) = columns_q.next()? {
            let column_name: String = column_q.get("name")?;
            let is_primary_key = column_q.get::<_, usize>("pk")? != 0;

            if is_primary_key {
                primary_key = Some(column_name.clone());
            }

            column_names.push(column_name);
        }

        let table = Table {
            escaped_name: escape_path(table_name),
            name: table_name.to_string(),

            // will be filled later
            create_table_sql: String::new(),
            create_index_sql: String::new(),

            columns: column_names,
            primary_key,
        };

        match tables_by_name.entry(table_name.to_string()) {
            Entry::Vacant(e) => {
                e.insert(table);
            },
            // Is this even possible?
            Entry::Occupied(_) => {
                return Err(Error::EdgeCase(String::from("table name collision")));
            },
        }
    }

    let mut sqls_q = sqls_stmt.query([])?;
    let mut sqls_by_table_name: HashMap<String, Vec<(String, String, String)>> = HashMap::new();

    while let Some(sql_q) = sqls_q.next()? {
        let r#type: String = sql_q.get("type")?;
        let object_name = sql_q.get("name")?;
        let table_name = sql_q.get("tbl_name")?;
        let sql: Option<String> = sql_q.get("sql")?;
        let sql = match sql {
            Some(sql) => sql,
            // AFAIK, auto indexes don't have `sql` field.
            // Since they're auto-generated, we don't have to care about them.
            None => { continue; },
        };

        match r#type.as_str() {
            "table" | "index" => {},
            _ => {
                return Err(Error::EdgeCase(format!("A type of a create script is `{type}`.")));
            },
        }

        match sqls_by_table_name.entry(table_name) {
            Entry::Occupied(mut e) => {
                e.get_mut().push((r#type, object_name, sql));
            },
            Entry::Vacant(e) => {
                e.insert(vec![(r#type, object_name, sql)]);
            },
        }
    }

    for (table_name, mut sqls) in sqls_by_table_name.into_iter() {
        match tables_by_name.get_mut(&table_name) {
            Some(table) => {
                // The result has to be deterministic, so that it doesn't confuse git.
                sqls.sort_by_key(|(_, name, _)| name.to_string());

                // If I sort it this way, `sqls[0]` must be `CREATE TABLE` and the
                // others must be `CREATE INDEX`
                sqls.sort_by_key(
                    |(t, _, _)| match t.as_str() {
                        "table" => 0,
                        "index" => 1,
                        _ => unreachable!(),
                    }
                );

                if sqls.len() == 0 || sqls[0].0 != "table" {
                    return Err(Error::EdgeCase(format!("no `CREATE TABLE` script found for `{table_name}`")));
                }

                if sqls.len() > 1 && sqls[1].0 == "table" {
                    return Err(Error::EdgeCase(format!("table `{table_name}` has more than 1 `CREATE TABLE` script")));
                }

                table.create_table_sql = sqls[0].2.clone();
                table.create_index_sql = (&sqls[1..]).iter().map(
                    // It seems like sqlite's dump doesn't end with ';' :(
                    |(_, _, sql)| format!("{sql};")
                ).collect::<Vec<_>>().join("\n");
            },
            None => {
                return Err(Error::EdgeCase(format!("There's a schema for table {table_name}, but there's no such table.")));
            },
        }
    }

    // As of now, there's no need to sort the result. But I'm sorting it anyway because
    // 1. It isn't that expensive.
    // 2. It's always safer to return a deterministic result.
    let mut result = tables_by_name.into_values().collect::<Vec<_>>();
    result.sort_by_key(|t| t.name.to_string());

    // AFAIK, auto-generated tables (sqlite_schema, sqlite_temp_schema) don't have
    // create sqls.
    result = result.into_iter().filter(|t| !t.create_table_sql.is_empty()).collect();

    Ok(result)
}

// TODO: make it configurable
const FLUSH_THRES: usize = 1024;

fn dump_db(
    db_path: &str,
    db_schema: &[Table],
    output_path: &str,
) -> Result<(), Error> {
    if exists(&output_path) {
        remove_dir_all(&output_path)?;
    }

    create_dir_all(&output_path)?;

    if db_schema.is_empty() {
        // There must be at least 1 file for git to do something.
        write_string(
            &join(output_path, ".empty")?,
            "",
            WriteMode::AlwaysCreate,
        )?;
        return Ok(());
    }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    for table in db_schema.iter() {
        let mut record_stmt = conn.prepare(&table.record_stmt())?;
        let mut records_q = record_stmt.query([])?;
        let mut records_by_id_prefix = HashMap::new();

        while let Some(record) = records_q.next()? {
            let mut fields = Vec::with_capacity(table.columns.len());

            // It's used to create the id of this record.
            let mut hash_data = Vec::with_capacity(
                if table.primary_key.is_some() {
                    1
                } else {
                    table.columns.len()
                }
            );

            for column_name in table.columns.iter() {
                let value: Value = record.get(column_name.as_str())?;

                match &table.primary_key {
                    Some(pk) if pk == column_name => {
                        hash_data.push(value.clone());
                    },
                    Some(_) => {},
                    None => {
                        hash_data.push(value.clone());
                    },
                }

                fields.push((column_name.to_string(), value));
            }

            let id = RecordId::hash(&hash_data);

            match records_by_id_prefix.entry(id.prefix()) {
                Entry::Occupied(mut e) => {
                    let v: &mut Vec<Record> = e.get_mut();
                    v.push(Record {
                        id,
                        fields,
                    });

                    // TODO: make this number configurable
                    if v.len() >= FLUSH_THRES {
                        flush(output_path, &table.escaped_name, id.prefix(), &v)?;
                        v.clear();
                    }
                },
                Entry::Vacant(e) => {
                    let mut v = Vec::with_capacity(FLUSH_THRES);
                    v.push(Record {
                        id,
                        fields,
                    });
                    e.insert(v);
                },
            }
        }

        for (prefix, records) in records_by_id_prefix.into_iter() {
            flush(output_path, &table.escaped_name, prefix, &records)?;
        }

        // TODO: dump table.create_table_sql and table.create_index_sql.
        let data_dir = join(output_path, &table.escaped_name)?;

        if !exists(&data_dir) {
            create_dir_all(&data_dir)?;
        }

        write_string(
            &join(
                &data_dir,
                "table.sql",
            )?,
            &table.create_table_sql,
            WriteMode::AlwaysCreate,
        )?;
        write_string(
            &join(
                &data_dir,
                "index.sql",
            )?,
            &table.create_index_sql,
            WriteMode::AlwaysCreate,
        )?;
    }

    Ok(())
}

fn flush(
    output_path: &str,
    table_name: &str,
    id_prefix: u64,
    records: &[Record],
) -> Result<(), Error> {
    let id_prefix_s = format!("{id_prefix:03o}");
    let data_dir = join(output_path, table_name)?;

    if !exists(&data_dir) {
        create_dir_all(&data_dir)?;
    }

    let data_path = join(&data_dir, &id_prefix_s)?;
    let mut data = if exists(&data_path) {
        read_records(&data_path)?
    } else {
        vec![]
    };

    data.append(&mut records.to_vec());
    data.sort_by_key(|r| r.id);
    write_records(&data_path, &data)?;
    Ok(())
}
