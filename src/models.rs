
// CREATE TABLE accounts (
//  guid text(32) PRIMARY KEY NOT NULL, 
// name text(2048) NOT NULL, 
// account_type text(2048) NOT NULL, 
// commodity_guid text(32), 
// commodity_scu integer NOT NULL, 
// non_std_scu integer NOT NULL, 
// parent_guid text(32), 
// code text(2048), 
// description text(2048), 
// hidden integer, 
// placeholder integer);


#[derive(Queryable)]
pub struct Accounts {
    pub guid: String,
    pub name: String,
    pub account_type: String,
    pub commodity_guid: String,
    pub commodity_scu: i32,
    pub non_std_scu: i32,
    pub parent_guid: String,
    pub code: String,
    pub description: String,
    pub hidden: i32,
    pub placeholder: i32
}

