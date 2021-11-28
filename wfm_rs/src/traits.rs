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

pub trait OrderID {
    fn order_id(&self) -> &str;
}

impl OrderID for String {
    fn order_id(&self) -> &str {
        self.as_str()
    }
}

impl OrderID for &str {
    fn order_id(&self) -> &str {
        self
    }
}

pub trait ItemID {
    fn item_id(&self) -> &str;
}

impl ItemID for String {
    fn item_id(&self) -> &str {
        self.as_str()
    }
}

impl ItemID for &str {
    fn item_id(&self) -> &str {
        self
    }
}
