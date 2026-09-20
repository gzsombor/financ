use anyhow::{Context, Result};
use diesel::prelude::*;

use crate::{
    cli::CommoditiesArgs,
    models::{Commodities, CommodityInfo},
};

pub struct CommoditiesQuery {
    pub limit: i64,
    pub name_filter: Option<String>,
    pub type_filter: Option<String>,
}

impl CommoditiesQuery {
    pub fn execute(&self, connection: &mut SqliteConnection) -> Result<Vec<Commodities>> {
        use crate::schema::commodities::dsl::{commodities, mnemonic, namespace};

        let mut query = commodities.into_boxed();
        if let Some(ref name_txt) = self.name_filter {
            query = query.filter(mnemonic.like(format!("%{name_txt}%")));
        }
        if let Some(ref type_txt) = self.type_filter {
            query = query.filter(namespace.like(format!("%{type_txt}%")));
        }

        query
            .limit(self.limit)
            .load::<Commodities>(connection)
            .context("Error loading commodities")
    }
    pub fn execute_and_display(&self, connection: &mut SqliteConnection) -> Result<usize> {
        let results = self.execute(connection)?;
        println!("Displaying {} commodities", results.len());
        let len = results.len();
        for commodity in results {
            commodity.display();
        }
        Ok(len)
    }

    pub fn get_by_guid(connection: &mut SqliteConnection, id: &str) -> Result<Option<Commodities>> {
        use crate::schema::commodities::dsl::{commodities, guid};

        Ok(commodities
            .filter(guid.eq(id))
            .limit(1)
            .load::<Commodities>(connection)
            .context("Error loading a commodity")?
            .pop())
    }

    pub fn get_info_by_guid(
        connection: &mut SqliteConnection,
        id: &str,
    ) -> Result<Option<CommodityInfo>> {
        Ok(Self::get_by_guid(connection, id)?.map(|c| CommodityInfo {
            guid: c.guid,
            mnemonic: c.mnemonic,
            fullname: c.fullname,
        }))
    }
}

impl From<CommoditiesArgs> for CommoditiesQuery {
    fn from(args: CommoditiesArgs) -> Self {
        CommoditiesQuery {
            limit: args.limit.unwrap_or(10),
            name_filter: args.name,
            type_filter: args.commodity_type,
        }
    }
}
