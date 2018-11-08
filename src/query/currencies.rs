use std::io;

use clap::ArgMatches;
use diesel::prelude::*;

use models::Commodities;

pub struct CommoditiesQuery {
    pub limit: i64,
    pub name_filter: Option<String>,
    pub type_filter: Option<String>,
}

impl CommoditiesQuery {
    pub fn execute(&self, connection: &SqliteConnection) -> Vec<Commodities> {
        use schema::commodities::dsl::*;

        let mut query = commodities.into_boxed();
        if let Some(ref name_txt) = self.name_filter {
            query = query.filter(mnemonic.like(format!("%{}%", name_txt)));
        }
        if let Some(ref type_txt) = self.type_filter {
            query = query.filter(namespace.like(format!("%{}%", type_txt)));
        }

        query
            .limit(self.limit)
            .load::<Commodities>(connection)
            .expect("Error loading commodities")
    }
    /*
        pub fn to_map(&self, connection: &SqliteConnection) -> BTreeMap<String, Commodities> {
            let mut commodity_map = BTreeMap::new();
            let results = self.execute(&connection);
            for commodity in results {
                commodity_map.insert(commodity.guid.clone(), commodity);
            }
            commodity_map
        }
    */
    pub fn execute_and_display(&self, connection: &SqliteConnection) -> io::Result<usize> {
        let results = self.execute(&connection);
        println!("Displaying {} commodities", results.len());
        let len = results.len();
        for commodity in results {
            commodity.display();
        }
        Ok(len)
    }

    pub fn get_by_guid(connection: &SqliteConnection, id: &str) -> Option<Commodities> {
        use schema::commodities::dsl::*;

        commodities
            .filter(guid.eq(id))
            .limit(1)
            .load::<Commodities>(connection)
            .expect("Error loading a commodity")
            .pop()
    }
}

impl<'a> From<&'a ArgMatches<'a>> for CommoditiesQuery {
    fn from(ls_acc_cmd: &ArgMatches) -> Self {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name_filter = value_t!(ls_acc_cmd, "name", String).ok();
        let type_filter = value_t!(ls_acc_cmd, "commodity-type", String).ok();
        CommoditiesQuery {
            limit,
            name_filter,
            type_filter,
        }
    }
}
