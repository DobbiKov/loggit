use crate::logger::file_handler::file_manager;
use crate::Level;

#[test]
fn parse_rotation_type() {
    let res = file_manager::RotationType::try_from_string("dfsa week".to_string());
    assert_eq!(res, None);

    let res = file_manager::RotationType::try_from_string("23 week".to_string());
    assert_eq!(
        res,
        Some(file_manager::RotationType::Period(60 * 60 * 24 * 7 * 23))
    )
}
