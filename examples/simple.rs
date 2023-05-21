#[cfg(all(feature = "derive", feature = "alloc"))]

use alkahest::{alkahest, serialize_to_vec, deserialize};

// Define simple formula. Make it self-serializable.
#[derive(Clone, Debug, PartialEq, Eq)]
#[alkahest(Formula, SerializeRef, Deserialize)]
struct MyDataType {
  a: u32,
  b: Vec<u8>,
}
fn main() {
  // Prepare data to serialize.
  let value = MyDataType {
    a: 1,
    b: vec![2, 3],
  };

  // Use infallible serialization to `Vec`.
  let mut data = Vec::new();

  // Note that this value can be serialized by reference.
  // This is default behavior for `Serialized` derive macro.
  // Some types required ownership transfer for serialization.
  // Notable example is iterators.
  let (size, _) = serialize_to_vec::<MyDataType, _>(&value, &mut data);

  let de = deserialize::<MyDataType, MyDataType>(&data[..size]).unwrap();
  assert_eq!(de, value);
  println!("Serialized size: {size} | data: {:?}", &de);
}