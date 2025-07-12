use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Property {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub units: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: i32,
    pub building_id: i32,
    pub tenant_id: Option<i32>,
    pub unit_number: String,
    pub is_occupied: bool,
}

#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,
}

pub struct Transaction {
    pub id: i32,
    pub property_id: Option<i32>,
    pub transaction_type: String,
    pub amount: f32,
    pub date: NaiveDate,
    pub description: String,
}