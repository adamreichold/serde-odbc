/*
This file is part of serde-odbc.

serde-odbc is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

serde-odbc is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with serde-odbc.  If not, see <http://www.gnu.org/licenses/>.
*/
use std::cell::RefCell;
use std::collections::HashMap;

use actix_web::{http::Method, server, App, HttpRequest, HttpResponse, Json};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Todo {
    text: String,
    done: bool,
}

#[derive(Clone, Default, Serialize)]
struct PersistentTodo {
    id: serde_odbc::Nullable<i32>,
    text: serde_odbc::String<typenum::U4096>,
    done: bool,
}

struct Service {
    conn: serde_odbc::Connection,
    select_all: serde_odbc::Statement<serde_odbc::NoParams, serde_odbc::RowSet<PersistentTodo>>,
    select_one: serde_odbc::Statement<serde_odbc::Params<i32>, serde_odbc::Cols<PersistentTodo>>,
    insert: serde_odbc::Statement<serde_odbc::Params<PersistentTodo>, serde_odbc::NoCols>,
    last_rowid: serde_odbc::Statement<serde_odbc::NoParams, serde_odbc::Cols<i32>>,
    update: serde_odbc::Statement<serde_odbc::Params<(PersistentTodo, i32)>, serde_odbc::NoCols>,
}

fn to_string<N: generic_array::ArrayLength<u8>>(value: &serde_odbc::String<N>) -> String {
    String::from_utf8(value.as_slice().unwrap().into()).unwrap()
}

impl Service {
    fn new(conn_str: &str) -> Result<Self, serde_odbc::Error> {
        let env = serde_odbc::Environment::new()?;
        let conn = serde_odbc::Connection::new(&env, conn_str)?;

        let trans = conn.begin();

        let mut create: serde_odbc::Statement<serde_odbc::NoParams, serde_odbc::NoCols> =
            serde_odbc::Statement::new(
                &conn,
                r"
                    CREATE TABLE IF NOT EXISTS todos (
                        id INTEGER PRIMARY KEY,
                        text VARCHAR(4096) NOT NULL,
                        done TINYINT NOT NULL
                    )
                ",
            )?;

        create.exec()?;

        trans.commit()?;

        let select_all =
            serde_odbc::Statement::with_fetch_size(&conn, "SELECT id, text, done FROM todos", 32)?;

        let select_one =
            serde_odbc::Statement::new(&conn, "SELECT id, text, done FROM todos WHERE id = ?")?;

        let insert = serde_odbc::Statement::new(
            &conn,
            "INSERT INTO todos (id, text, done) VALUES (?, ?, ?)",
        )?;

        let last_rowid = serde_odbc::Statement::new(&conn, "SELECT last_insert_rowid()")?;

        let update = serde_odbc::Statement::new(
            &conn,
            "UPDATE todos SET id = ?, text = ?, done = ? WHERE id = ?",
        )?;

        Ok(Service {
            conn,
            select_all,
            select_one,
            insert,
            last_rowid,
            update,
        })
    }

    fn get_todos(&mut self) -> Result<HashMap<i32, Todo>, serde_odbc::Error> {
        let trans = self.conn.begin();
        let stmt = &mut self.select_all;

        stmt.exec()?;

        let mut todos = HashMap::new();

        while stmt.fetch()? {
            todos.reserve(stmt.cols().len());
            todos.extend(stmt.cols().iter().map(|todo| {
                (
                    *todo.id.get().unwrap(),
                    Todo {
                        text: to_string(&todo.text),
                        done: todo.done,
                    },
                )
            }));
        }

        trans.commit()?;

        Ok(todos)
    }

    fn get_todo(&mut self, id: i32) -> Result<Option<Todo>, serde_odbc::Error> {
        let trans = self.conn.begin();
        let stmt = &mut self.select_one;

        *stmt.params() = id;

        stmt.exec()?;

        let found = stmt.fetch()?;

        trans.commit()?;

        if !found {
            return Ok(None);
        }

        Ok(Some(Todo {
            text: to_string(&stmt.cols().text),
            done: stmt.cols().done,
        }))
    }

    fn add_todo(&mut self, todo: &Todo) -> Result<i32, serde_odbc::Error> {
        let trans = self.conn.begin();
        let stmt = &mut self.insert;

        stmt.params().text.assign(todo.text.as_bytes());
        stmt.params().done = todo.done;

        stmt.exec()?;

        let stmt = &mut self.last_rowid;

        stmt.exec()?;

        let id = if stmt.fetch()? { *stmt.cols() } else { -1 };

        trans.commit()?;

        Ok(id)
    }

    fn set_todo(&mut self, id: i32, todo: &Todo) -> Result<(), serde_odbc::Error> {
        let trans = self.conn.begin();
        let stmt = &mut self.update;

        stmt.params().1 = id;
        stmt.params().0.id.assign(id);
        stmt.params().0.text.assign(todo.text.as_bytes());
        stmt.params().0.done = todo.done;

        stmt.exec()?;

        trans.commit()?;

        Ok(())
    }
}

fn handle_req<T: Serialize, H: FnOnce(&mut Service) -> Result<T, serde_odbc::Error>>(
    req: HttpRequest<RefCell<Service>>,
    handler: H,
) -> HttpResponse {
    match handler(&mut req.state().borrow_mut()) {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(err) => HttpResponse::InternalServerError().body(format!("{:?}", err)),
    }
}

fn get_todos(req: HttpRequest<RefCell<Service>>) -> HttpResponse {
    handle_req(req, Service::get_todos)
}

fn get_todo(req: HttpRequest<RefCell<Service>>) -> HttpResponse {
    let id = req.match_info().query("id").unwrap();

    handle_req(req, |svc| svc.get_todo(id))
}

fn add_todo((todo, req): (Json<Todo>, HttpRequest<RefCell<Service>>)) -> HttpResponse {
    handle_req(req, |svc| svc.add_todo(&todo))
}

fn set_todo((todo, req): (Json<Todo>, HttpRequest<RefCell<Service>>)) -> HttpResponse {
    let id = req.match_info().query("id").unwrap();

    handle_req(req, |svc| svc.set_todo(id, &todo))
}

fn main() {
    let bind_addr = "127.0.0.1:8080";
    let conn_str = "Driver=Sqlite3;Database=todos.db;";

    println!("Listening on {}...", bind_addr);

    server::new(move || {
        App::with_state(RefCell::new(Service::new(conn_str).unwrap()))
            .route("/todos", Method::GET, get_todos)
            .route("/todo/{id}", Method::GET, get_todo)
            .route("/todos", Method::POST, add_todo)
            .route("/todo/{id}", Method::POST, set_todo)
    })
    .bind(bind_addr)
    .unwrap()
    .run();
}
