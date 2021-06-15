lalrpop_mod!(pub grammar);

#[cfg(test)]
mod lalrpop_tests {
    use super::*;

    #[test]
    pub fn test_ideal_record_number_parse() {
        // Input string
        let input = "<test_key:5:N>";

        // Parse the input
        let result = grammar::RecordValueParser::new().parse(input);

        if result.is_err(){
            println!("{:?}", result);
        }
        assert!(result.is_ok());
        let result = result.unwrap();

        // Check actual values
        assert_eq!("TEST_KEY", result.0);
        assert_eq!(5, result.1);
        assert_eq!(Some('N'), result.2);
    }
}
