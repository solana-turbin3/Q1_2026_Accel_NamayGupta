use borsh::{BorshDeserialize, BorshSerialize, from_slice, to_vec};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::PhantomData;
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig, deserialize, serialize};
pub trait Serializer<T> {
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>>;
    fn from_bytes(&self, data: &[u8]) -> Result<T, Box<dyn Error>>;
}
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Borsh;
impl<T> Serializer<T> for Borsh
where
    T: BorshSerialize + BorshDeserialize,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        to_vec(data).map_err(|e| e.into())
    }
    fn from_bytes(&self, data: &[u8]) -> Result<T, Box<dyn Error>> {
        from_slice(data).map_err(|e| e.into())
    }
}
#[derive(SchemaWrite, SchemaRead)]
pub struct Wincode;
impl<T> Serializer<T> for Wincode
where
    T: SchemaWrite<DefaultConfig, Src = T> + for<'de> SchemaRead<'de, DefaultConfig, Dst = T>,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        serialize(data).map_err(|e| e.into())
    }
    fn from_bytes(&self, data: &[u8]) -> Result<T, Box<dyn Error>> {
        deserialize(data).map_err(|e| e.into())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Json;
impl<T> Serializer<T> for Json
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    fn to_bytes(&self, data: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        serde_json::to_vec(data).map_err(|e| e.into())
    }
    fn from_bytes(&self, data: &[u8]) -> Result<T, Box<dyn Error>> {
        serde_json::from_slice(data).map_err(|e| e.into())
    }
}

pub struct GeneralStorage<T, S> {
    data: Vec<u8>,
    serializer: S,
    phantom: PhantomData<T>,
}

impl<T, S> GeneralStorage<T, S>
where
    S: Serializer<T>,
{
    fn new(serializer: S) -> Self {
        Self {
            data: Vec::new(),
            serializer,
            phantom: PhantomData,
        }
    }
    fn save(&mut self, value: &T) -> Result<(), Box<dyn Error>> {
        self.data = self.serializer.to_bytes(value)?;
        Ok(())
    }
    fn load(&self) -> Result<T, Box<dyn Error>> {
        self.serializer.from_bytes(&self.data)
    }
    fn has_data(&self) -> bool {
        !self.data.is_empty()
    }
    fn convert<S2: Serializer<T>>(
        &mut self,
        new_serializer: S2,
    ) -> Result<GeneralStorage<T, S2>, Box<dyn Error>> {
        let mut new_storage = GeneralStorage::new(new_serializer);
        new_storage.save(&self.load()?)?;

        Ok(new_storage)
    }
}
#[derive(
    Debug,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    SchemaWrite,
    SchemaRead,
)]
pub struct TestData {
    pub name: String,
    pub age: u32,
}

#[test]
fn test_general_storage_borsh() {
    let mut borsh_storage: GeneralStorage<TestData, Borsh> = GeneralStorage::new(Borsh);
    let test_data = TestData {
        name: "Namay".to_string(),
        age: 24,
    };
    borsh_storage.save(&test_data).expect("saving failed");
    let loaded = borsh_storage.load().unwrap();
    assert_eq!(loaded, test_data);
    assert!(borsh_storage.has_data());
}
#[test]
fn test_general_storage_json() {
    let mut json_storage: GeneralStorage<TestData, Json> = GeneralStorage::new(Json);
    let test_data = TestData {
        name: "Namay".to_string(),
        age: 24,
    };
    json_storage.save(&test_data).expect("saving failed");
    let loaded = json_storage.load().unwrap();
    assert_eq!(loaded, test_data);
    assert!(json_storage.has_data());
}
#[test]
fn test_general_storage_wincode() {
    let mut wincode_storage: GeneralStorage<TestData, Wincode> = GeneralStorage::new(Wincode);
    let test_data = TestData {
        name: "Namay".to_string(),
        age: 24,
    };
    wincode_storage.save(&test_data).expect("saving failed");
    let loaded = wincode_storage.load().unwrap();
    assert_eq!(loaded, test_data);
    assert!(wincode_storage.has_data());
}
#[test]
fn test_general_storage_convert() {
    let mut json_storage: GeneralStorage<TestData, Json> = GeneralStorage::new(Json);
    let test_data = TestData {
        name: "Namay".to_string(),
        age: 24,
    };
    json_storage.save(&test_data).expect("saving failed");
    let loaded1 = json_storage.load().unwrap();
    assert_eq!(loaded1, test_data);
    assert!(json_storage.has_data());
    let new_borsh_storage = json_storage.convert(Borsh).expect("converting failed");

    let loaded2 = new_borsh_storage.load().unwrap();
    assert_eq!(loaded2, test_data);
    assert!(json_storage.has_data())
}
