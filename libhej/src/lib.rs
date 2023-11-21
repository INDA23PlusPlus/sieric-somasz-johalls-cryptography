use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Get {
    Id(String),
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Put {
    Id(String),
    Data(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serial() {
        let s = String::from("HEllo");
        let get = Get::Id(s);
        let serialied = serde_yaml::to_string(&get).unwrap();
        let deserialized: Get = serde_yaml::from_str(&serialied).unwrap();
        println!("{:?}", serialied);
        println!("{:?}", deserialized);

        assert!(true);
    }
}
