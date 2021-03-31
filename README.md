# Djanco extensions

This crate defines the `djanco` macro. The macro points at functions which 
contain Djanco queries and specifies the configuration of the database that
should be used for a particular query. The tags are used by 
![cargo-djanco](https://github.com/PRL-PRG/cargo-djanco) to generate an 
execution harness for all the queries in a given cargo crate.

The macro provided by `djanco_ext` is called `djanco` and looks like this:

```rust
#[djanco(April, 2020, subsets(C, Python, SmallProjects))]
pub fn my_query(database: &Database, _log: &Log, output: &Path) -> Result<(), std::io::Error>  {
    database.projects()
        .group_by(project::Language)
        .sort_by(project::Stars)
        .sample(Top(1000))
        .into_csv_in_dir(output, "top_1000_by_stars.csv")
}
```

The macro specifies a number of arguments that configure the date (savepoint)
at which the database will checked out and the substores from which the data 
will be read. See ![cargo-djanco](https://github.com/PRL-PRG/cargo-djanco) 
for details.
