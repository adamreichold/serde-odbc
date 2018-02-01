extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_odbc;
extern crate typenum;

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use futures::prelude::*;

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
    insert: serde_odbc::Statement<serde_odbc::Params<PersistentTodo>, serde_odbc::NoCols>,
}

#[derive(Clone)]
struct ServiceHandle(Rc<RefCell<Service>>);

impl Service {
    fn new(conn_str: &str) -> Self {
        let env = serde_odbc::Environment::new().unwrap();
        let conn = serde_odbc::Connection::new(&env, conn_str).unwrap();

        {
            let mut stmt: serde_odbc::Statement<
                serde_odbc::NoParams,
                serde_odbc::NoCols,
            > = serde_odbc::Statement::new(
                &conn,
                r#"
                    CREATE TABLE todos (
                        id INTEGER PRIMARY KEY,
                        text VARCHAR(4096) NOT NULL,
                        done TINYINT NOT NULL
                    )
                "#,
            ).unwrap();
            stmt.exec().unwrap();
        }

        let mut select_all =
            serde_odbc::Statement::new(&conn, "SELECT id, text, done FROM todos").unwrap();
        serde_odbc::FetchSize::set_fetch_size(&mut select_all, 32);

        let insert = serde_odbc::Statement::new(
            &conn,
            "INSERT INTO todos (id, text, done) VALUES (?, ?, ?)",
        ).unwrap();

        Service {
            conn,
            select_all,
            insert,
        }
    }

    fn do_select_all(&mut self, todos: &mut Vec<Todo>) {
        let trans = self.conn.begin();
        let stmt = &mut self.select_all;

        stmt.exec().unwrap();

        while stmt.fetch().unwrap() {
            todos.reserve(stmt.cols().len());
            todos.extend(stmt.cols().iter().map(|todo| Todo {
                text: String::from_utf8(todo.text.as_slice().unwrap().into()).unwrap(),
                done: todo.done,
            }));
        }

        trans.commit().unwrap();
    }

    fn do_insert(&mut self, todo: &Todo) {
        let trans = self.conn.begin();
        let stmt = &mut self.insert;

        stmt.params().text.assign(todo.text.as_bytes());
        stmt.params().done = todo.done;

        stmt.exec().unwrap();

        trans.commit().unwrap();
    }

    fn get_todos(&mut self) -> Vec<u8> {
        let mut todos = Vec::new();

        self.do_select_all(&mut todos);

        serde_json::to_vec(&todos).unwrap()
    }

    fn post_todo(&mut self, todo: &[u8]) {
        let todo = serde_json::from_slice(todo).unwrap();

        self.do_insert(&todo);
    }
}

impl ServiceHandle {
    fn new(conn_str: &str) -> Self {
        ServiceHandle(Rc::new(RefCell::new(Service::new(conn_str))))
    }

    fn get_mut(&self) -> RefMut<Service> {
        self.0.borrow_mut()
    }
}

impl hyper::server::Service for ServiceHandle {
    type Request = hyper::server::Request;
    type Response = hyper::server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&hyper::Method::Get, "/todos") => {
                let todos = self.get_mut().get_todos();

                Box::new(futures::future::ok(
                    hyper::server::Response::new()
                        .with_header(hyper::header::ContentType::json())
                        .with_header(hyper::header::ContentLength(todos.len() as u64))
                        .with_body(todos),
                ))
            }
            (&hyper::Method::Post, "/todos") => {
                let svc = self.clone();

                Box::new(req.body().concat2().map(move |body| {
                    svc.get_mut().post_todo(body.as_ref());

                    hyper::server::Response::new()
                }))
            }
            _ => Box::new(futures::future::ok(
                hyper::server::Response::new().with_status(hyper::StatusCode::NotFound),
            )),
        }
    }
}

fn main() {
    let svc = ServiceHandle::new("Driver=Sqlite3;Database=:memory:;");

    let server = hyper::server::Http::new()
        .bind(&([127, 0, 0, 1], 8080).into(), move || Ok(svc.clone()))
        .unwrap();

    println!("Listening on 127.0.0.1:8080...");

    server.run().unwrap();
}
