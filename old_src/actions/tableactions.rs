

use controller::repository::Repository;
use controller::types::DataPoint;
use controller::schema::Schema;
use controller::crud;
use controller::rows::Rows;
use controller::query::GetQuery;

/*

TODO: screen actions:
- range: start to end, start to rest
- filter: where clause
- sort: asc, desc
# immutable
- select: column filter
- join
- map: row operation, output to another column
- shift: move entire column
- reduce:
- groupby: ???

*/

enum Selector {
    GetAll,
    GetSection { start: u64, end: u64 },
    GetRest { start: u64 },
    // TODO: get one item by key,
}

impl Selector {
    pub fn get_query(&self) -> GetQuery {
        match self {
            Selector::GetAll => GetQuery::all(),
            Selector::GetSection { start, end } => GetQuery::new().offset(start).limit(end - start),
            Selector::GetRest { start } => GetQuery::new().offset(start),
        }
    }
}


trait Modifications {

}

trait Deletions {

}

trait NewData {

}


#[derive(Clone)]
enum Error {

}

trait Actions<MyType> {
    fn replace(&self, new_data: &NewData) -> Result<MyType, Error>;
    fn update(&self, modifications: &Modifications) -> Result<MyType, Error>;
    fn delete(&self, selector: &Deletions) -> Result<MyType, Error>;
    fn retrieve(&self, selector: &Selector) -> Result<Rows, Error>;
    fn count(&self) -> Result<u64, Error>;
}

#[derive(Clone)]
struct TableActions {
    repository: Box<Repository>,
    table_id: String,
    user: i8,
}

impl TableActions {
    fn new(repository: Box<Repository>, table_id: &String) -> Self {
        TableActions {
            repository,
            table_id: table_id.to_owned(),
            user: 0,
        }
    }
}

impl Actions<TableActions> for TableActions {


    fn replace(&self, new_data: &NewData) -> Result<Self, Error> {
        Ok(self.to_owned())
    }

    fn update(&self, modifications: &Modifications) -> Result<Self, Error> {
        Ok(self.to_owned())
    }

    fn delete(&self, selector: &Deletions) -> Result<Self, Error> {
        Ok(self.to_owned())
    }

    fn retrieve(&self, selector: &Selector) -> Result<Rows, Error> {
        let TableActions { repository, table_id, .. } = self;
        let repository_ref = Box::leak(repository.to_owned());
        let schema = crud::get_table_schema(repository_ref, &table_id);

        let table = crud::get_table_rows(repository_ref, &table_id, null);


        Ok(Rows::new(&vec![]))
    }

    fn count(&self) -> Result<u64, Error> {
        Ok(0)
    }
}


