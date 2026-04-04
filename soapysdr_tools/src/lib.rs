pub fn soapysdr_tools_works() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = soapysdr_tools_works();
        assert_eq!(result, true);
    }
}
