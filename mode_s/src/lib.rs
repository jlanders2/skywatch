pub fn mode_s_works() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = mode_s_works();
        assert_eq!(result, true);
    }
}
