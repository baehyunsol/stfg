# A tool that converts sqlite to/from git-friendly format

This is a tool made by me and made for me, to solve a very specific problem: version-control sqlite.

I love git and sqlite. Both are very reliable and feature-rich. I use them a lot. One problem is that it's very difficult to version-control an sqlite DB with git. You can add the database file to the git repository, but git's not very good at dealing with binary files. You can't `git diff` binary files, you can't `git blame` them. Also, git works better with many small files than a single huge file.

So I made this tool. I'm satisfied with it, and I hope you like it too!

## How to install

1. Build from source

```sh
git clone https://github.com/baehyunsol/stfg
cd stfg
cargo build --release
```

2. Build with cargo

```sh
# I haven't published yet. I'll publish this soon!
# cargo install stfg
```

3. Use pre-built binaries

Coming soon!

## How to use

1. `git commit` your database

```sh
# This will create a directory `db/` and dump the data to the directory.
# If `db/` already exists, it removes files in the directory. So be careful!
stfg to-git your-database.db -o db/

# Then, run whatever git command you want.
git add db
git commit
```

2. `git checkout` older version of your database

```sh
git checkout older-version-of-your-database

# Let's assume that `db/` is an output of `stfg to-git` command.
# It overwrites `your-database.db` if it already exists.
stfg from-git db/ -o your-database.db

# Now `your-database.db` contains an older version of your data.
```

3. `git diff` between 2 versions of your database

```sh
# Let's assume that `db/` is an output of `stfg to-git` command.
# stfg creates a subdirectory per table under `db/`, so you can
# easily browse the contents of each table.
git diff HEAD~1:db/your_table_name HEAD:db/your_table_name
```

## FAQ

1. Why not just use `.dump` command of sqlite?

Yeah that works, and many of you might find that better than my solution. But I think my solution is more git-friendly because 1) it splits the database into small files instead of creating a single huge file and 2) it guarantees each field of the database is exactly one line in a file.

2. Can I use this in production?

I don't think so. It's my personal project. There must be a lot of rough edges. Feedbacks and contributions are welcome!!
