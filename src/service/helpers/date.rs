use chrono::{NaiveDate, ParseResult};

pub fn parse_date(month: i32, year: i32) -> ParseResult<NaiveDate> {
    let date_str = format!("{}-{}-01", year, month);
    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
}

#[cfg(test)]
mod test {
    use crate::service::helpers::date::parse_date;

    #[test]
    fn test_parse_date() {
        let res = parse_date(2, 2020).unwrap();
        assert_eq!(res.to_string(), "2020-02-01")
    }
}
