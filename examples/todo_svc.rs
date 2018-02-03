extern crate futures;
extern crate generic_array;
extern crate hyper;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_odbc;
extern crate typenum;

use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::collections::HashMap;

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

#[derive(Debug)]
enum Error {
    SerdeJson(serde_json::Error),
    SerdeOdbc(serde_odbc::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}

impl From<serde_odbc::Error> for Error {
    fn from(err: serde_odbc::Error) -> Self {
        Error::SerdeOdbc(err)
    }
}

struct Service {
    conn: serde_odbc::Connection,
    select_all: serde_odbc::Statement<serde_odbc::NoParams, serde_odbc::RowSet<PersistentTodo>>,
    select_one: serde_odbc::Statement<serde_odbc::Params<i32>, serde_odbc::Cols<PersistentTodo>>,
    insert: serde_odbc::Statement<serde_odbc::Params<PersistentTodo>, serde_odbc::NoCols>,
    update: serde_odbc::Statement<serde_odbc::Params<(PersistentTodo, i32)>, serde_odbc::NoCols>,
    match_path: regex::Regex,
}

#[derive(Clone)]
struct ServiceHandle(Rc<RefCell<Service>>);

fn to_string<N: generic_array::ArrayLength<u8>>(value: &serde_odbc::String<N>) -> String {
    String::from_utf8(value.as_slice().unwrap().into()).unwrap()
}

impl Service {
    fn new(conn_str: &str) -> Result<Self, serde_odbc::Error> {
        let env = serde_odbc::Environment::new()?;
        let conn = serde_odbc::Connection::new(&env, conn_str)?;

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

        let mut select_all = serde_odbc::Statement::new(&conn, "SELECT id, text, done FROM todos")?;
        serde_odbc::FetchSize::set_fetch_size(&mut select_all, 32);

        let select_one =
            serde_odbc::Statement::new(&conn, "SELECT id, text, done FROM todos WHERE id = ?")?;

        let insert = serde_odbc::Statement::new(
            &conn,
            "INSERT INTO todos (id, text, done) VALUES (?, ?, ?)",
        )?;

        let update = serde_odbc::Statement::new(
            &conn,
            "UPDATE todos SET id = ?, text = ?, done = ? WHERE id = ?",
        )?;

        let match_path = regex::Regex::new(r"/todos/(?P<id>\d+)").unwrap();

        Ok(Service {
            conn,
            select_all,
            select_one,
            insert,
            update,
            match_path,
        })
    }

    fn do_select_all(&mut self) -> Result<HashMap<i32, Todo>, serde_odbc::Error> {
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

    fn do_select_one(&mut self, id: i32) -> Result<Option<Todo>, serde_odbc::Error> {
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

    fn do_insert(&mut self, todo: &Todo) -> Result<(), serde_odbc::Error> {
        let trans = self.conn.begin();
        let stmt = &mut self.insert;

        stmt.params().text.assign(todo.text.as_bytes());
        stmt.params().done = todo.done;

        stmt.exec()?;

        trans.commit()?;

        Ok(())
    }

    fn do_update(&mut self, id: i32, todo: &Todo) -> Result<(), serde_odbc::Error> {
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

    fn get_todos(&mut self) -> Result<Vec<u8>, Error> {
        let todos = self.do_select_all()?;
        let todos = serde_json::to_vec(&todos)?;

        Ok(todos)
    }

    fn get_todo(&mut self, id: i32) -> Result<Vec<u8>, Error> {
        let todo = self.do_select_one(id)?;
        let todo = serde_json::to_vec(&todo)?;

        Ok(todo)
    }

    fn add_todo(&mut self, todo: &[u8]) -> Result<(), Error> {
        let todo = serde_json::from_slice(todo)?;
        self.do_insert(&todo)?;

        Ok(())
    }

    fn set_todo(&mut self, id: i32, todo: &[u8]) -> Result<(), Error> {
        let todo = serde_json::from_slice(todo)?;
        self.do_update(id, &todo)?;

        Ok(())
    }
}

impl ServiceHandle {
    fn new(conn_str: &str) -> Result<Self, serde_odbc::Error> {
        Ok(ServiceHandle(Rc::new(RefCell::new(Service::new(
            conn_str,
        )?))))
    }

    fn get_mut(&self) -> RefMut<Service> {
        self.0.borrow_mut()
    }
}

fn response_with_status(status: hyper::StatusCode) -> hyper::server::Response {
    hyper::server::Response::new().with_status(status)
}

fn response_with_body(body: Vec<u8>) -> hyper::server::Response {
    hyper::server::Response::new()
        .with_header(hyper::header::ContentType::json())
        .with_header(hyper::header::ContentLength(body.len() as u64))
        .with_body(body)
}

fn response_from_result(body: Result<Vec<u8>, Error>) -> hyper::server::Response {
    match body {
        Ok(todos) => response_with_body(todos),
        Err(_) => response_with_status(hyper::StatusCode::InternalServerError),
    }
}

fn boxed_response(
    resp: hyper::server::Response,
) -> Box<futures::Future<Item = hyper::server::Response, Error = hyper::Error>> {
    Box::new(futures::future::ok(resp))
}

fn await_body<F: FnOnce(&mut Service, &[u8]) -> Result<(), Error> + 'static>(
    svc: &ServiceHandle,
    req: hyper::server::Request,
    f: F,
) -> Box<futures::Future<Item = hyper::server::Response, Error = hyper::Error>> {
    use futures::{Future, Stream};

    let svc = svc.clone();

    Box::new(req.body().concat2().map(move |body| {
        let status = match f(&mut svc.get_mut(), body.as_ref()) {
            Ok(()) => hyper::StatusCode::Ok,
            Err(_) => hyper::StatusCode::InternalServerError,
        };
        response_with_status(status)
    }))
}

impl hyper::server::Service for ServiceHandle {
    type Request = hyper::server::Request;
    type Response = hyper::server::Response;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let mut svc = self.get_mut();

        if req.path() == "/todos" {
            if req.method() == &hyper::Method::Get {
                return boxed_response(response_from_result(svc.get_todos()));
            }

            if req.method() == &hyper::Method::Post {
                return await_body(self, req, |svc, body| svc.add_todo(body));
            }
        }

        let id: Option<i32> = svc.match_path.captures(req.path()).map(|caps| caps["id"].parse().unwrap());

        if let Some(id) = id {
            if req.method() == &hyper::Method::Get {
                return boxed_response(response_from_result(svc.get_todo(id)));
            }

            if req.method() == &hyper::Method::Post {
                return await_body(self, req, move |svc, body| svc.set_todo(id, body));
            }
        }

        boxed_response(response_with_status(hyper::StatusCode::NotFound))
    }
}

fn main() {
    let svc = ServiceHandle::new("Driver=Sqlite3;Database=todos.db;").unwrap();

    let server = hyper::server::Http::new()
        .bind(&([127, 0, 0, 1], 8080).into(), move || Ok(svc.clone()))
        .unwrap();

    println!("Listening on 127.0.0.1:8080...");

    server.run().unwrap();
}
