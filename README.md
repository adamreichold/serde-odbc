[![Build Status](https://travis-ci.org/adamreichold/serde-odbc.svg?branch=master)](https://travis-ci.org/adamreichold/serde-odbc)

Bind serializable Rust data to ODBC statements
----------------------------------------------

The main function of this crate is to use the `Serialize` trait to automatically make the necessary calls to `SQLBindCol` and `SQLBindParameter`. It also supports binding of parameter and row sets, e.g. the following code performs a bulk insert:
```
#[derive(Clone, Default, Serialize)]
struct Todo {
    id: serde_odbc::Nullable<i32>,
    text: serde_odbc::String<typenum::U4096>,
    done: bool,
}

let stmt: serde_odbc::Statement<serde_odbc::ParamSet<Todo>, serde_odbc::NoCols> =
    serde_odbc::Statement::new(&conn, "INSERT INTO todos (id, text, done) VALUES (?, ?, ?)");

stmt.params().reserve(128);
for todo in /* ... */ {
    stmt.params().push(todo);
}

stmt.exec().unwrap();
```
