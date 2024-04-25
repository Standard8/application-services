pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

uniffi::include_scaffolding!("search");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
