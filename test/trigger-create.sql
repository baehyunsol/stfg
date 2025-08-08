CREATE TABLE student (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    age INTEGER NOT NULL CONSTRAINT age_constraint CHECK (age BETWEEN 1 AND 150),
    major TEXT,
    is_old INTEGER DEFAULT 0
);

CREATE TRIGGER insert_old_student
    AFTER INSERT ON student
    FOR EACH ROW WHEN NEW.age > 30
    BEGIN
        UPDATE student SET is_old = 1 WHERE student.id = NEW.id;
    END;

CREATE TRIGGER update_old_student
    AFTER UPDATE ON student
    FOR EACH ROW WHEN NEW.age > 30
    BEGIN
        UPDATE student SET is_old = 1 WHERE student.id = NEW.id;
    END;
