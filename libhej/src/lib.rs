use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Get {
    pub id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Put {
    pub id: String,
    pub data: String,
}
#[derive(Serialize, Deserialize, Debug)]

pub enum MessageType {
    Put(Put),
    Get(Get),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serial() {
        let s = String::from("HEllo");
        let get = Get { id: s };
        let serialied = serde_yaml::to_string(&get).unwrap();
        let deserialized: Get = serde_yaml::from_str(&serialied).unwrap();
        println!("{:?}", serialied);
        println!("{:?}", deserialized);

        assert!(true);
    }
}
