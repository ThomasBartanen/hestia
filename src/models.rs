#[derive(Debug, Clone)]
pub struct Building {
    pub id: i32,
    pub name: String,    
    pub units: Vec<Unit>,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: i32,
    pub building_id: i32,
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