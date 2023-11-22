use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Get {
    pub id: String,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Put {
    pub id: String,
    pub data: String,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MessageType {
    Put(Put),
    Get(Get),
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GetResponse {
    pub data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serial() {
        let s = String::from("HEllo");
        let get = Get { id: s };
        let serialied = serde_json::to_string(&get).unwrap();
        let deserialized: Get = serde_json::from_str(&serialied).unwrap();
        println!("{:?}", serialied);
        println!("{:?}", deserialized);

        assert!(1+1 == 3);
    }
}
