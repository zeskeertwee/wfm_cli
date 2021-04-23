pub trait ItemUrl {
    fn item_url(&self) -> &str;
}

impl ItemUrl for String {
    fn item_url(&self) -> &str {
        self.as_str()
    }
}

impl ItemUrl for &str {
    fn item_url(&self) -> &str {
        self
    }
}