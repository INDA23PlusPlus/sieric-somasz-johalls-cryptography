# Server
```rust
fn send_for_verification() -> hash {
    // Returns the top hash of the merkle tree    

}

fn send_data(id) -> (id, data){
    // Sends the data stored at the ID
}
    
fn store_data(id, data){
    //It runs on a post request and stores the data with the 
} 

```


# Client

```rust

fn send_data(password, id, file) ->  (id, SignedData){
    //It encypts and signes the file and it saves the encyption key wit hthe file ID

}
fn sign_data(key, id, EncyptedData) -> SignedData {
    // Data in our case is the encypted data of the file
}


fn genrate_key(password, id) -> key {
    //Ganarets an encryption key according to the specifiactions of the HACKmd
}

fn encytp_file(key file) -> EncyptedData{
    //Encrypts according to the specification in the HACKmd
}

fn decrypt_data(id, data) -> file {
    //Uses the id to find the encryption key and returns the decypted file

}

fn verify_data_integrity(hash) -> bool{
    // Compares the merkle tree hash from the serve rwith the clients hash to confirm if the data is up to data or not.
}
```

