-- I asked claude sonnet 3.7 to generate an SQL script that creates edge cases and he gave me this.
-- I did very small modifications to the code and it works!

-- Start with a transaction for atomicity
BEGIN TRANSACTION;

-- =============================================
-- 1. Tables with various column types and constraints
-- =============================================

-- Basic table with many data types and constraints
CREATE TABLE test_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    text_col TEXT NOT NULL,
    int_col INTEGER DEFAULT 42,
    real_col REAL CHECK(real_col > 0),
    blob_col BLOB,
    null_col NULL,
    "column with spaces" TEXT,
    "quoted""column" TEXT,
    [bracketed column] INTEGER,
    `backtick_column` TEXT,
    -- SQLite allows any type name, not just the standard ones
    custom_type_col mycustomtype,
    -- Unicode column name
    "колонка" TEXT,
    -- Emoji in column name
    "😀_emoji_col" TEXT
);

-- Table with unusual PRIMARY KEY definition
CREATE TABLE composite_key (
    part1 TEXT,
    part2 INTEGER,
    data BLOB,
    PRIMARY KEY (part2, part1) -- Reversed order, composite
);

-- Table with generated columns (SQLite 3.31.0+)
CREATE TABLE generated_columns (
    id INTEGER PRIMARY KEY,
    first_name TEXT,
    last_name TEXT,
    full_name TEXT GENERATED ALWAYS AS (first_name || ' ' || last_name) STORED,
    name_length INTEGER GENERATED ALWAYS AS (length(full_name)) VIRTUAL
);

-- Table with all possible constraints
CREATE TABLE all_constraints (
    id INTEGER CONSTRAINT pk_constraint PRIMARY KEY,
    unique_col TEXT CONSTRAINT unique_constraint UNIQUE,
    not_null_col INTEGER CONSTRAINT nn_constraint NOT NULL,
    check_col INTEGER CONSTRAINT check_constraint CHECK(check_col BETWEEN 1 AND 100),
    default_col TEXT CONSTRAINT default_constraint DEFAULT 'default value',
    collate_col TEXT COLLATE NOCASE,
    -- Make an FK reference to a table created earlier
    fk_col INTEGER CONSTRAINT fk_constraint REFERENCES test_types(id) ON DELETE CASCADE ON UPDATE RESTRICT
);

-- =============================================
-- 2. Tables with strange names
-- =============================================

-- Table with spaces in name
CREATE TABLE "Table With Spaces" (id INTEGER PRIMARY KEY, data TEXT);

-- Table with quotes in name
CREATE TABLE "Table""With""Quotes" (id INTEGER PRIMARY KEY, data TEXT);

-- Table with reserved keyword as name (needs quoting)
CREATE TABLE "select" (id INTEGER PRIMARY KEY, "from" TEXT);

-- Table with unicode name
CREATE TABLE "таблица" (id INTEGER PRIMARY KEY, data TEXT);

-- Table with emoji in name
CREATE TABLE "table_😀" (id INTEGER PRIMARY KEY, data TEXT);

-- =============================================
-- 3. Views
-- =============================================

-- Simple view
CREATE VIEW simple_view AS 
SELECT id, text_col FROM test_types;

-- View with complex query
CREATE VIEW complex_view AS
SELECT t.id, t.text_col, a.unique_col, 
       (SELECT COUNT(*) FROM composite_key) as count_subquery
FROM test_types t
JOIN all_constraints a ON t.id = a.fk_col
WHERE t.int_col > 10
GROUP BY t.id
HAVING COUNT(*) > 1
ORDER BY t.text_col;

-- View with unusual name
CREATE VIEW "view with ""quotes""" AS
SELECT * FROM "Table With Spaces";

-- =============================================
-- 4. Indexes
-- =============================================

-- Regular index
CREATE INDEX idx_test_types_text ON test_types(text_col);

-- Composite index
CREATE INDEX idx_composite ON test_types(text_col, int_col);

-- Unique index
CREATE UNIQUE INDEX idx_unique ON all_constraints(unique_col, not_null_col);

-- Descending order index
CREATE INDEX idx_desc ON test_types(int_col DESC);

-- Index with WHERE clause
CREATE INDEX idx_partial ON test_types(text_col) WHERE int_col > 0;

-- Index with function
CREATE INDEX idx_func ON test_types(lower(text_col));

-- Index with expression
CREATE INDEX idx_expr ON test_types(text_col || '-' || int_col);

-- Index with unusual name
CREATE INDEX "index with spaces" ON "Table With Spaces"(id);

-- =============================================
-- 5. Triggers
-- =============================================

-- Basic INSERT trigger
CREATE TRIGGER trg_after_insert AFTER INSERT ON test_types
BEGIN
    INSERT INTO "Table With Spaces"(data) VALUES (NEW.text_col);
END;

-- Multiple statement trigger
CREATE TRIGGER trg_complex BEFORE UPDATE ON all_constraints
FOR EACH ROW WHEN OLD.check_col <> NEW.check_col
BEGIN
    UPDATE "Table With Spaces" SET data = 'Updated' WHERE id = OLD.id;
    INSERT INTO composite_key(part1, part2) VALUES('trigger', NEW.id);
END;

-- Trigger with unusual name
CREATE TRIGGER "trigger with ""quotes""" AFTER DELETE ON test_types
BEGIN
    SELECT 1;
END;

-- =============================================
-- 6. Virtual tables
-- =============================================

-- FTS5 virtual table (full-text search)
CREATE VIRTUAL TABLE fts_docs USING fts5(
    title, 
    body, 
    tokenize='porter unicode61'
);

-- R-Tree virtual table (spatial index)
CREATE VIRTUAL TABLE rtree_test USING rtree(
    id,
    minX, maxX,
    minY, maxY
);

-- =============================================
-- 7. Insert test data with edge cases
-- =============================================

-- Regular data
INSERT INTO test_types (text_col, int_col, real_col, blob_col, "column with spaces", "quoted""column", [bracketed column], `backtick_column`, custom_type_col, "колонка", "😀_emoji_col") 
VALUES ('Regular text', 123, 45.67, X'DEADBEEF', 'spaces value', 'quotes"value', 42, 'backtick value', 'custom', 'юникод', 'emoji value');

-- Edge case: Empty string vs NULL
INSERT INTO test_types (text_col, int_col, real_col, blob_col, "column with spaces", "quoted""column")
VALUES ('', 0, 0.1, X'', '', '');

-- Edge case: Unicode and special characters
INSERT INTO test_types (text_col, int_col, real_col)
VALUES ('Unicode: 你好世界 Special chars: ¿¡@#$%^&*()_+', 42, 3.14159);

-- Edge case: Very long text
INSERT INTO test_types (text_col, int_col, real_col)
VALUES (replace(hex(randomblob(1000)), '0', 'A'), 42, 3.14159);

-- Edge case: Large integer values
INSERT INTO test_types (text_col, int_col, real_col)
VALUES ('Large int', 9223372036854775807, 3.14159);

-- Edge case: Scientific notation
INSERT INTO test_types (text_col, int_col, real_col)
VALUES ('Scientific', 42, 1.23e-45);

-- Edge case: Zero-length BLOB
INSERT INTO test_types (text_col, int_col, real_col, blob_col)
VALUES ('Empty BLOB', 42, 3.14159, X'');

-- Edge case: Large BLOB
INSERT INTO test_types (text_col, int_col, real_col, blob_col)
VALUES ('Large BLOB', 42, 3.14159, randomblob(100000));

-- Populate the FTS table
INSERT INTO fts_docs(title, body) VALUES('SQLite Tutorial', 'SQLite is a self-contained, embedded database engine.');
INSERT INTO fts_docs(title, body) VALUES('Edge Cases', 'Testing all the edge cases for SQLite dump tool.');

-- =============================================
-- 8. Foreign key relationships with cycles
-- =============================================

-- Create tables with circular references (need to be careful with constraint timing)
CREATE TABLE parent(
    id INTEGER PRIMARY KEY,
    child_id INTEGER,
    data TEXT
);

CREATE TABLE child(
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES parent(id),
    data TEXT
);

-- Add the circular reference after both tables exist
-- ALTER TABLE parent ADD CONSTRAINT fk_child FOREIGN KEY (child_id) REFERENCES child(id) DEFERRABLE INITIALLY DEFERRED;

-- Insert cyclic data (only works because the constraint is deferred)
INSERT INTO parent (id, data) VALUES (100, 'Parent data');
INSERT INTO child (id, parent_id, data) VALUES (200, 100, 'Child data');
UPDATE parent SET child_id = 200 WHERE id = 100;

-- =============================================
-- 9. Self-referencing table
-- =============================================

CREATE TABLE employees(
    id INTEGER PRIMARY KEY,
    name TEXT,
    manager_id INTEGER REFERENCES employees(id),
    department TEXT
);

INSERT INTO employees (id, name, manager_id, department) VALUES
(1, 'CEO', NULL, 'Executive'),
(2, 'CTO', 1, 'Technology'),
(3, 'Developer', 2, 'Technology');

-- =============================================
-- 10. Tables with unusual data storage
-- =============================================

-- WITHOUT ROWID table (more efficient for certain use cases)
CREATE TABLE without_rowid_table(
    id INTEGER,
    data TEXT,
    PRIMARY KEY (id)
) WITHOUT ROWID;

INSERT INTO without_rowid_table VALUES (1, 'Without rowid data');

-- Table with strict typing (SQLite 3.37.0+)
CREATE TABLE strict_types(
    id INTEGER PRIMARY KEY,
    text_only TEXT,
    int_only INTEGER,
    real_only REAL
) STRICT;

-- Try to insert data with correct types
INSERT INTO strict_types (text_only, int_only, real_only) 
VALUES ('text', 42, 3.14);

-- =============================================
-- 11. More advanced views
-- =============================================

-- Recursive CTE view
CREATE VIEW recursive_view AS
WITH RECURSIVE org_hierarchy(id, name, manager_id, level, path) AS (
    SELECT id, name, manager_id, 0, name
    FROM employees
    WHERE manager_id IS NULL
    UNION ALL
    SELECT e.id, e.name, e.manager_id, h.level + 1, h.path || ' > ' || e.name
    FROM employees e, org_hierarchy h
    WHERE e.manager_id = h.id
)
SELECT id, name, manager_id, level, path FROM org_hierarchy ORDER BY path;

-- View on a view
CREATE VIEW view_on_view AS
SELECT * FROM recursive_view WHERE level > 0;

-- =============================================
-- 12. Edge cases with DEFAULT values
-- =============================================

-- Table with unusual DEFAULT values
-- CREATE TABLE default_values (
--     id INTEGER PRIMARY KEY,
--     current_timestamp_default TEXT DEFAULT CURRENT_TIMESTAMP,
--     expression_default INTEGER DEFAULT (1+1),
--     function_default TEXT DEFAULT lower('HELLO')
-- );

-- INSERT INTO default_values (id) VALUES (1);

-- =============================================
-- 13. JSON and custom collations (if supported)
-- =============================================

-- JSON column (SQLite 3.9.0+)
CREATE TABLE json_table (
    id INTEGER PRIMARY KEY,
    json_data JSON,
    parsed_value TEXT GENERATED ALWAYS AS (json_extract(json_data, '$.name')) STORED
);

INSERT INTO json_table (json_data) VALUES ('{"name": "Test", "values": [1, 2, 3]}');
INSERT INTO json_table (json_data) VALUES ('{"name": null, "complex": {"nested": "value"}}');

-- =============================================
-- 14. Security edge cases
-- =============================================

-- Table with names that could be SQL injection vectors
CREATE TABLE "dummy); DROP TABLE users; --" (
    id INTEGER PRIMARY KEY,
    "'); DELETE FROM important_data; --" TEXT
);

INSERT INTO "dummy); DROP TABLE users; --" ("'); DELETE FROM important_data; --")
VALUES ('This is a security test');

-- =============================================
-- 15. Application-specific metadata
-- =============================================

-- Store the application version in user_version pragma
PRAGMA user_version = 42;

-- Create application metadata table
CREATE TABLE _app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT
);

INSERT INTO _app_metadata VALUES 
('schema_version', '1.0.0'),
('created_at', datetime('now')),
('created_by', 'SQLite Test Generator');

-- =============================================
-- 16. Temporary tables
-- =============================================

CREATE TEMPORARY TABLE temp_data (
    id INTEGER PRIMARY KEY,
    temporary_value TEXT
);

INSERT INTO temp_data VALUES (1, 'This is temporary');

-- =============================================
-- End transaction
-- =============================================

COMMIT;

-- Run VACUUM to compact the database (tests handling of this operation)
VACUUM;

-- Set some PRAGMA values that should be preserved
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
