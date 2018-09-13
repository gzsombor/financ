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

    pub fn execute_and_display(&self, connection: &SqliteConnection) {
        let results = self.execute(&connection);
        println!("Displaying {} commodities", results.len());
        for commodity in results {
            commodity.display();
        }
    }
}

impl<'a> From<&'a ArgMatches<'a>> for CommoditiesQuery {
    fn from(ls_acc_cmd: &ArgMatches) -> Self {
        let limit = value_t!(ls_acc_cmd, "limit", i64).unwrap_or(10);
        let name_filter = value_t!(ls_acc_cmd, "name", String).ok();
        let type_filter = value_t!(ls_acc_cmd, "type", String).ok();
        CommoditiesQuery {
            limit,
            name_filter,
            type_filter,
        }
    }
}
