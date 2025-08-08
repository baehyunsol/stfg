CREATE TABLE student (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    age INTEGER NOT NULL CONSTRAINT age_constraint CHECK (age BETWEEN 1 AND 150),
    major TEXT
);

CREATE VIEW student_by_major (major, count) AS SELECT major, COUNT(*) FROM student GROUP BY major;
CREATE VIEW student_by_age (age, count) AS SELECT age, COUNT(*) FROM student GROUP BY age;
