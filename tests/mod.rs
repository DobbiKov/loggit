use crate::Level;

#[test]
fn test_file_format_to_file_name() {
    let format = super::logger::file_handler::FileFormatter::try_from_string(
        "{time}fdsafdas{date}fdsa.txt".to_string(),
    )
    .unwrap();
    let temp_lvl = Level::INFO;
    let file_name =
        super::logger::file_handler::FileName::from_file_formatter(format, temp_lvl).unwrap();
    println!("{:?}", file_name);
    println!("{}", String::from(file_name));
}
