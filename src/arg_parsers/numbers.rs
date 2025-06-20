use std::str::FromStr;

/// Strips leading and trailing whitespace from an input string slice
/// and attempts to parse the remaining string into a specified numeric type `T`.
///
/// # Arguments
///
/// * `input`: A string slice (`&str`).
///
/// # Type Parameters
///
/// * `T`: The target numeric type. This type must implement `std::str::FromStr`.
///        The associated error type `T::Err` should implement `Debug` for easier error handling.
///
/// # Returns
///
/// * `Ok(T)`: If stripping and parsing are successful, containing the parsed number.
/// * `Err(T::Err)`: If parsing fails, containing the error from `T::from_str`.
/// ```
pub fn strip_and_parse_number<T>(input: &str) -> Result<T, T::Err>
where
    T: FromStr,         // The target type T must be parsable from a string.
    T::Err: std::error::Error,
{
    let trimmed_str = input.trim();
    trimmed_str.parse::<T>()
}

#[cfg(test)]
mod tests {
    use super::*; // Import the function from the parent module

    #[test]
    fn test_parse_i32_valid() {
        assert_eq!(strip_and_parse_number::<i32>("  123  "), Ok(123));
        assert_eq!(strip_and_parse_number::<i32>("  -456\t"), Ok(-456));
        assert_eq!(strip_and_parse_number::<i32>("0"), Ok(0));
    }

    #[test]
    fn test_parse_f64_valid() {
        assert_eq!(strip_and_parse_number::<f64>("  3.14159  "), Ok(3.14159));
        assert_eq!(strip_and_parse_number::<f64>("\n-0.5\r\n"), Ok(-0.5));
    }

    #[test]
    fn test_parse_usize_valid() {
        assert_eq!(strip_and_parse_number::<usize>("  9876543210  "), Ok(9876543210));
    }

    #[test]
    fn test_parse_string_slice_input() {
        let input_string = String::from("  777  ");
        assert_eq!(strip_and_parse_number::<i32>(&input_string), Ok(777)); // Pass as &str
    }

    #[test]
    fn test_parse_invalid_integer() {
        let result = strip_and_parse_number::<i32>("  123.45  "); // Floats are not ints
        assert!(result.is_err());

        let result_text = strip_and_parse_number::<i32>("  hello  ");
        assert!(result_text.is_err());
    }

    #[test]
    fn test_parse_invalid_float() {
        let result = strip_and_parse_number::<f64>("  3..14  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_after_trim() {
        let result = strip_and_parse_number::<i32>("    ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_input() {
        let result = strip_and_parse_number::<i32>("");
        assert!(result.is_err());
    }
}