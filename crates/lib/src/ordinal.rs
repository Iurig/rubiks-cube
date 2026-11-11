pub fn ordinal(number: usize) -> String {
    let ending = match number % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{number}{ending}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinals_match_1_2_3_4() {
        assert_eq!("1st", &ordinal(1));
        assert_eq!("2nd", &ordinal(2));
        assert_eq!("3rd", &ordinal(3));
        assert_eq!("4th", &ordinal(4));
    }
}
