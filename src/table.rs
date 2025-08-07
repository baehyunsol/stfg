#[derive(Clone, Debug)]
pub struct Table {
    // In most cases `name == escaped_name`.
    // If `name` contains a character that cannot be used in paths (SQL table name
    // can have an arbitrary character!), it'd escaped.
    pub escaped_name: String,

    pub name: String,
    pub create_table_sql: String,
    pub create_index_sql: String,

    // We only need names of the columns because all the necessary information
    // to create the columns can be found in `create_scripts`.
    // All we need is the names so that we can read the values. We don't even
    // need the types because 1) sqlite DB is usually loosely-typed and 2)
    // stfg uses a dynamic-typed object to read the values.
    pub columns: Vec<String>,

    // This affects how stfg creates an id of a record.
    pub primary_key: Option<String>,
}

impl Table {
    pub fn record_stmt(&self) -> String {
        format!(
            "SELECT {} FROM '{}';",
            self.columns.iter().map(
                |column| format!("\"{}\"", column.replace("\"", "\"\""))
            ).collect::<Vec<_>>().join(", "),
            self.name.replace("'", "''"),
        )
    }

    pub fn insert_stmt(&self) -> String {
        format!(
            "INSERT INTO '{}' ({}) VALUES ({})",
            self.name.replace("'", "''"),
            self.columns.iter().map(
                |column| format!("'{}'", column.replace("'", "''"))
            ).collect::<Vec<_>>().join(", "),
            self.columns.iter().enumerate().map(
                |(n, _)| format!("?{}", n + 1)
            ).collect::<Vec<_>>().join(", "),
        )
    }
}

// It converts `s` into a string that's safe to use in file names.
pub(crate) fn escape_path(s: &str) -> String {
    let mut chars = vec![];

    for ch in s.chars() {
        match ch {
            '0'..='9'
            | 'a'..='z'
            | 'A'..='Z'
            | '가'..='힣'
            | '_' | '-' | '.' => {
                chars.push(ch);
            },
            _ => {
                let mut d = vec![0; 4];
                let mut hexes = vec![];
                let l = ch.encode_utf8(&mut d).len();

                // Nothing's much special about '$'.
                // It's safe to use in file names in all Operating Systems (AFAIK),
                // and I like dollars.
                chars.push('$');

                for b in d[0..l].iter() {
                    hexes.push(format!("{b:02x}"));
                }

                for c in hexes.concat().chars() {
                    chars.push(c);
                }

                chars.push('$');
            },
        }
    }

    chars.into_iter().collect()
}
