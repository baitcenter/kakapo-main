
pub mod results;
pub mod error;
pub mod channels;

use actix::prelude::*;

use serde_json;

use std::result::Result;
use std::result::Result::Ok;

use diesel::{r2d2::ConnectionManager, r2d2::PooledConnection};
use diesel::pg::PgConnection;

use data;

use connection::py::PyRunner;

use model::entity;
use model::entity::RetrieverFunctions;
use model::entity::ModifierFunctions;
use model::entity::error::DBError;

use model::schema;

use model::actions::results::*;
use model::actions::error::Error;
use model::actions::results::NamedActionResult;
use data::utils::OnDuplicate;
use model::entity::results::Upserted;
use model::entity::results::Created;
use data::utils::OnNotFound;
use serde::Serialize;
use model::entity::conversion;
use model::dbdata::RawEntityTypes;
use model::entity::Modifier;

type State = PooledConnection<ConnectionManager<PgConnection>>; //TODO: should include user data

pub trait Action: Send
    where
        Self::Ret: Send + serde::Serialize + NamedActionResult
{
    type Ret;
    fn call(&self, state: &State/*, session: Session*/) -> Result<Self::Ret, Error>;
}

///decorator for permission
pub struct WithPermissionRequired<A: Action> {
    action: A,
    //permission: ...
}

impl<A: Action> WithPermissionRequired<A> {
    pub fn new(action: A/*, permission: Permission*/) -> Self {
        Self {
            action,
            //permission,
        }
    }
}

impl<A: Action> Action for WithPermissionRequired<A> {
    type Ret = A::Ret; //TODO: 403 error
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        self.action.call(state)
    }
}

///decorator for permission in listing items
pub struct WithFilterListByPermission<A: Action> {
    action: A,
    //permission: ...
}

///decorator for transactions
pub struct WithTransaction<A: Action> {
    action: A,
    //permission: ...
}

impl<A: Action> WithTransaction<A> {
    pub fn new(action: A) -> Self {
        Self {
            action,
        }
    }
}

impl<A: Action> Action for WithTransaction<A> {
    type Ret = A::Ret;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //TODO: transactions
        self.action.call(state)
    }
}

///get all tables
#[derive(Debug)]
pub struct GetAllTables {
    pub show_deleted: bool,
}

impl GetAllTables {
    pub fn new(show_deleted: bool) -> impl Action<Ret = GetAllTablesResult> {
        Self {
            show_deleted,
        }
    }
}

impl Action for GetAllTables {
    type Ret = GetAllTablesResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let entities: Vec<data::Table> = entity::Retriever::get_all(&state)
            .or_else(|err| Err(Error::DB(err)))?;
        Ok(GetAllTablesResult(entities))
    }
}

///get all queries
#[derive(Debug)]
pub struct GetAllQueries {
    pub show_deleted: bool,
}

impl GetAllQueries {
    pub fn new(show_deleted: bool) -> impl Action<Ret = GetAllQueriesResult> {
        Self {
            show_deleted,
        }
    }
}

impl Action for GetAllQueries {
    type Ret = GetAllQueriesResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let entities: Vec<data::Query> = entity::Retriever::get_all(&state)
            .or_else(|err| Err(Error::DB(err)))?;
        Ok(GetAllQueriesResult(entities))
    }
}

///get all scripts
#[derive(Debug)]
pub struct GetAllScripts {
    pub show_deleted: bool,
}

impl GetAllScripts {
    pub fn new(show_deleted: bool) -> impl Action<Ret = GetAllScriptsResult> {
        Self {
            show_deleted,
        }
    }
}

impl Action for GetAllScripts {
    type Ret = GetAllScriptsResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let entities: Vec<data::Script> = entity::Retriever::get_all(&state)
            .or_else(|err| Err(Error::DB(err)))?;
        Ok(GetAllScriptsResult(entities))
    }
}

///get one table
#[derive(Debug)]
pub struct GetTable {
    pub name: String,
    //pub detailed: bool, //TODO: is this needed?
}

impl GetTable {
    pub fn new(name: String) -> impl Action<Ret = GetTableResult> { //weird syntax but ok
        let action = Self {
            name,
        };
        let action_with_transaction = WithTransaction::new(action);
        let action_with_permission = WithPermissionRequired::new(action_with_transaction /*, ... */);

        action_with_permission
    }
}

impl Action for GetTable {
    type Ret = GetTableResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let maybe_entity: Option<data::Table> = entity::Retriever::get_one(&state, &self.name)
            .or_else(|err| Err(Error::DB(err)))?;

        match maybe_entity {
            Some(entity) => Ok(GetTableResult(entity)),
            None => Err(Error::NotFound),
        }
    }
}

///get one query
#[derive(Debug)]
pub struct GetQuery {
    pub name: String,
}

impl GetQuery {
    pub fn new(name: String) -> impl Action<Ret = GetQueryResult> { //weird syntax but ok
        let action = Self {
            name,
        };
        let action_with_transaction = WithTransaction::new(action);
        let action_with_permission = WithPermissionRequired::new(action_with_transaction /*, ... */);

        action_with_permission
    }
}

impl Action for GetQuery {
    type Ret = GetQueryResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let maybe_entity: Option<data::Query> = entity::Retriever::get_one(&state, &self.name)
            .or_else(|err| Err(Error::DB(err)))?;

        match maybe_entity {
            Some(entity) => Ok(GetQueryResult(entity)),
            None => Err(Error::NotFound),
        }
    }
}

///get one script
#[derive(Debug)]
pub struct GetScript {
    pub name: String,
}

impl GetScript {
    pub fn new(name: String) -> impl Action<Ret = GetScriptResult> { //weird syntax but ok
        let action = Self {
            name,
        };
        let action_with_transaction = WithTransaction::new(action);
        let action_with_permission = WithPermissionRequired::new(action_with_transaction /*, ... */);

        action_with_permission
    }
}

impl Action for GetScript {
    type Ret = GetScriptResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        let maybe_entity: Option<data::Script> = entity::Retriever::get_one(&state, &self.name)
            .or_else(|err| Err(Error::DB(err)))?;

        match maybe_entity {
            Some(entity) => Ok(GetScriptResult(entity)),
            None => Err(Error::NotFound),
        }
    }
}

///create one table
#[derive(Debug)]
pub struct CreateEntity<T>
    where
        T: Serialize + Send + Clone,
        T: RawEntityTypes,
        T: conversion::GenerateRaw<<T as RawEntityTypes>::NewData>,
{
    pub data: T,
    pub on_duplicate: OnDuplicate,
}

impl<T> CreateEntity<T>
    where
        T: Serialize + Send + Clone,
        T: RawEntityTypes,
        T: conversion::GenerateRaw<<T as RawEntityTypes>::NewData>,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            on_duplicate: OnDuplicate::Ignore,
        }
    }
}

impl<T> Action for CreateEntity<T>
    where
        CreateEntityResult<T>: NamedActionResult,
        T: Serialize + Send + Clone,
        T: RawEntityTypes,
        T: conversion::GenerateRaw<<T as RawEntityTypes>::NewData>,
        Modifier: ModifierFunctions<T, T>,
{
    type Ret = CreateEntityResult<T>;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        match &self.on_duplicate {
            OnDuplicate::Update => {
                entity::Modifier::upsert(&state, self.data.clone())
                    .or_else(|err| Err(Error::DB(err)))
                    .and_then(|res| {
                        match res {
                            Upserted::Update { old, new } => Ok(new),
                            Upserted::Create { new } => Ok(new),
                        }
                    })
            },
            OnDuplicate::Ignore => {
                entity::Modifier::create(&state, self.data.clone())
                    .or_else(|err| Err(Error::DB(err)))
                    .and_then(|res| {
                        match res {
                            Created::Success { new } => Ok(new),
                            Created::Fail { old, .. } => Ok(old),
                        }
                    })

            },
            OnDuplicate::Fail => {
                entity::Modifier::create(&state, self.data.clone())
                    .or_else(|err| Err(Error::DB(err)))
                    .and_then(|res| {
                        match res {
                            Created::Success { new } => Ok(new),
                            Created::Fail { .. } => Err(Error::AlreadyExists),
                        }
                    })
            },
        }.and_then(|res| Ok(CreateEntityResult::<T>(res)))
    }
}

///update table
#[derive(Debug)]
pub struct UpdateTable {
    pub name: String,
    pub data: data::Table,
    pub on_not_found: OnNotFound,
}

impl UpdateTable {
    pub fn new(name: String, data: data::Table) -> impl Action<Ret = CreateEntityResult<data::Table>> {
        Self {
            name,
            data,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for UpdateTable {
    type Ret = CreateEntityResult<data::Table>;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::update_table(&state, self.name.to_owned(), self.reqdata.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

///update query
#[derive(Debug)]
pub struct UpdateQuery {
    pub name: String,
    pub data: data::Query,
    pub on_not_found: OnNotFound,
}

impl UpdateQuery {
    pub fn new(name: String, data: data::Query) -> impl Action<Ret = CreateEntityResult<data::Query>> {
        Self {
            name,
            data,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for UpdateQuery {
    type Ret = CreateEntityResult<data::Query>;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::update_query(&state, self.name.to_owned(), self.reqdata.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

///update script
#[derive(Debug)]
pub struct UpdateScript {
    pub name: String,
    pub data: data::Script,
    pub on_not_found: OnNotFound,
}

impl UpdateScript {
    pub fn new(name: String, data: data::Script) -> impl Action<Ret = CreateEntityResult<data::Script>> {
        Self {
            name,
            data,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for UpdateScript {
    type Ret = CreateEntityResult<data::Script>;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::update_script(&state, self.name.to_owned(), self.reqdata.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

///delete table
#[derive(Debug)]
pub struct DeleteTable {
    pub name: String,
    pub on_not_found: OnNotFound,
}

impl DeleteTable {
    pub fn new(name: String) -> impl Action<Ret = ()> {
        Self {
            name,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for DeleteTable {
    type Ret = ();
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::delete_table(&state, self.name.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

///delete query
#[derive(Debug)]
pub struct DeleteQuery {
    pub name: String,
    pub on_not_found: OnNotFound,
}

impl DeleteQuery {
    pub fn new(name: String) -> impl Action<Ret = ()> {
        Self {
            name,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for DeleteQuery {
    type Ret = ();
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::delete_query(&state, self.name.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

///delete script
#[derive(Debug)]
pub struct DeleteScript {
    pub name: String,
    pub on_not_found: OnNotFound,
}

impl DeleteScript {
    pub fn new(name: String) -> impl Action<Ret = ()> {
        Self {
            name,
            on_not_found: OnNotFound::Ignore,
        }
    }
}

impl Action for DeleteScript {
    type Ret = ();
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = manage::create::delete_script(&state, self.name.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

// Table Actions
#[derive(Debug)]
pub struct QueryTableData {
    pub name: String,
    //pub start: Option<usize>,
    //pub end: Option<usize>,
    //pub format: api::TableDataFormat,
}

impl QueryTableData {
    pub fn new(name: String) -> impl Action<Ret = GetTableDataResult> {
        Self {
            name
        }
    }
}

impl Action for QueryTableData {
    type Ret = GetTableDataResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = table::get_table_data(&state, self.name.to_owned(), self.format.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}


#[derive(Debug)]
pub struct InsertTableData {
    pub name: String,
    //pub data: api::TableData, //payload
    //pub format: api::TableDataFormat,
    //pub on_duplicate: api::OnDuplicate,
}

impl InsertTableData {
    pub fn new(name: String) -> impl Action<Ret = InsertTableDataResult> {
        Self {
            name
        }
    }
}

impl Action for InsertTableData {
    type Ret = InsertTableDataResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = table::insert_table_data(
        //    &state,
        //    self.name.to_owned(), self.data.to_owned(), self.format.to_owned(), api::CreationMethod::default())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

#[derive(Debug)]
pub struct UpdateTableData {
    pub name: String,
    //pub key: String,
    //pub data: api::RowData, //payload
    //pub format: api::TableDataFormat,
}

impl UpdateTableData {
    pub fn new(name: String) -> impl Action<Ret = UpdateTableDataResult> {
        Self {
            name
        }
    }
}

impl Action for UpdateTableData {
    type Ret = UpdateTableDataResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = table::update_table_data(
        //    &state,
        //    self.name.to_owned(), self.key.to_owned(), self.data.to_owned(), self.format.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

#[derive(Debug)]
pub struct DeleteTableData {
    pub name: String,
    //pub key: String,
}

impl DeleteTableData {
    pub fn new(name: String) -> impl Action<Ret = DeleteTableDataResult> {
        Self {
            name
        }
    }
}

impl Action for DeleteTableData {
    type Ret = DeleteTableDataResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = table::delete_table_data(&state, self.name.to_owned(), self.key.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

// Query Action
#[derive(Debug)]
pub struct RunQuery {
    pub name: String,
    //pub params: api::QueryParams,
    //pub start: Option<usize>,
    //pub end: Option<usize>,
    //pub format: api::TableDataFormat,
}

impl RunQuery {
    pub fn new(name: String) -> impl Action<Ret = RunQueryResult> {
        Self {
            name
        }
    }
}

impl Action for RunQuery {
    type Ret = RunQueryResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = query::run_query(
        //    &state,
        //    self.name.to_owned(), self.format.to_owned(), self.params.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

// Query Action
#[derive(Debug)]
pub struct RunScript {
    pub name: String,
    //pub params: api::ScriptParam,
}

impl RunScript {
    pub fn new(name: String) -> impl Action<Ret = RunScriptResult> {
        Self {
            name
        }
    }
}

impl Action for RunScript {
    type Ret = RunScriptResult;
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        //let result = script::run_script(
        //    &state,
        //    self.py_runner.to_owned(), self.name.to_owned(), self.params.to_owned())
        //    .or_else(|err| Err(()))?;
        //Ok(result)
        Err(Error::Unknown)
    }
}

//Other utitlies
#[derive(Debug)]
pub struct Nothing;

impl Nothing {
    pub fn new() -> impl Action<Ret = ()> {
        Nothing
    }
}

impl Action for Nothing {
    type Ret = ();
    fn call(&self, state: &State) -> Result<Self::Ret, Error> {
        Ok(())
    }
}