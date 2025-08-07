import os
import shutil
import sqlite3
import subprocess
import sys

test_name = sys.argv[1]

# if create_script exists, it resets the db
db_path = f"{test_name}.db"
db_path2 = f"{test_name}-2.db"

create_script = f"{test_name}-create.sql"
insert_script = f"{test_name}-insert.sql"
delete_script = f"{test_name}-delete.sql"

if os.path.exists(create_script):
    if os.path.exists(db_path):
        os.remove(db_path)

    conn = sqlite3.connect(db_path)

    with open(create_script, "r") as f:
        conn.executescript(f.read())

    conn.commit()

if os.path.exists(insert_script):
    with open(insert_script, "r") as f:
        insert_query = f.read()

else:
    insert_query = None

if os.path.exists(delete_script):
    with open(delete_script, "r") as f:
        delete_query = f.read()

else:
    delete_query = None

# initialize git
subprocess.run(["cargo", "run", "--release", "--", "to-git", db_path, "-o", "test-dir/db/"], check=True)
subprocess.run(["git", "-C", "test-dir", "init"], check=True)
subprocess.run(["git", "-C", "test-dir", "add", "--all"], check=True)
subprocess.run(["git", "-C", "test-dir", "commit", "-m", "test"], check=True)

# re-construct the sqlite db from the git data
subprocess.run(["cargo", "run", "--release", "--", "from-git", "test-dir/db", "-o", db_path2], check=True)

# re-construct the git data with the new db
subprocess.run(["cargo", "run", "--release", "--", "to-git", db_path2, "-o", "test-dir/db/"], check=True)
subprocess.run(["git", "-C", "test-dir", "add", "--all"], check=True)

# It must fail because I haven't updated the db.
assert subprocess.run(["git", "-C", "test-dir", "commit", "-m", "test"]).returncode != 0

os.remove(db_path2)

# re-construct sqlite db from the git data, again
subprocess.run(["cargo", "run", "--release", "--", "from-git", "test-dir/db", "-o", db_path2], check=True)

# update the db and commit the changes to git
if insert_query is not None:
    conn = sqlite3.connect(db_path2)
    conn.executescript(insert_query)
    conn.commit()
    subprocess.run(["cargo", "run", "--release", "--", "to-git", db_path2, "-o", "test-dir/db/"], check=True)
    subprocess.run(["git", "-C", "test-dir", "add", "--all"], check=True)

    # there must be something to commit
    subprocess.run(["git", "-C", "test-dir", "commit", "-m", "test"], check=True)

if delete_query is not None:
    conn = sqlite3.connect(db_path2)
    conn.executescript(delete_query)
    conn.commit()
    subprocess.run(["cargo", "run", "--release", "--", "to-git", db_path2, "-o", "test-dir/db/"], check=True)
    subprocess.run(["git", "-C", "test-dir", "add", "--all"], check=True)

    # there must be something to commit
    subprocess.run(["git", "-C", "test-dir", "commit", "-m", "test"], check=True)

# run from-git and to-git again. There should be no changes.
if insert_query is not None or delete_query is not None:
    subprocess.run(["cargo", "run", "--release", "--", "from-git", "test-dir/db", "-o", db_path2], check=True)
    subprocess.run(["cargo", "run", "--release", "--", "to-git", db_path2, "-o", "test-dir/db/"], check=True)
    subprocess.run(["git", "-C", "test-dir", "add", "--all"], check=True)
    assert subprocess.run(["git", "-C", "test-dir", "commit", "-m", "test"]).returncode != 0

shutil.rmtree("test-dir")
