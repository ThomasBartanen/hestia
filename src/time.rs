use chrono::{Duration, NaiveDate};

pub fn get_today() -> NaiveDate {
    return chrono::Utc::now().date_naive();
}

pub fn get_naivedate_x_days_ago(days: i64) -> NaiveDate {
    return get_today() - Duration::days(days);
}
