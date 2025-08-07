CREATE TABLE "a/b" (
    id INTEGER PRIMARY KEY,
    "c/d" INTEGER
);
INSERT INTO "a/b" (id, "c/d") VALUES (1, 2);

CREATE TABLE "a'b" (
    id INTEGER PRIMARY KEY,
    "c'd" INTEGER
);
INSERT INTO "a'b" (id, "c'd") VALUES (3, 4);

CREATE TABLE "a""b" (
    id INTEGER PRIMARY KEY,
    "c""d" INTEGER
);
INSERT INTO "a""b" (id, "c""d") VALUES (5, 6);
