use std::io::Read;
use std::fmt::{ Display, Result, Formatter};
struct ByteVector(Vec<u8>);

impl Display for ByteVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[ ")?;
        for v in &self.0 {
            write!(f, "{}, ", v)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

fn read_version(bytes_slice: &mut &[u8]) -> u32 {
    println!("bytes_slice memory address before reading {:p}", *bytes_slice);
    // Read slice into a buffer
    let mut buffer = [0; 4];
    bytes_slice.read(&mut buffer).unwrap();
    println!("version bytes: {:?}", buffer);

    println!("bytes_slice memory address after reading {:p}\n", *bytes_slice);
    println!("bytes_slice: {:?}\n", bytes_slice);

    u32::from_le_bytes(buffer)
}

fn main() {
    let transaction_hex = "010000000242d5c1d6f7308bbe95c0f6e1301dd73a8da77d2155b0773bc297ac47f9cd7380010000006a4730440220771361aae55e84496b9e7b06e0a53dd122a1425f85840af7a52b20fa329816070220221dd92132e82ef9c133cb1a106b64893892a11acf2cfa1adb7698dcdc02f01b0121030077be25dc482e7f4abad60115416881fe4ef98af33c924cd8b20ca4e57e8bd5feffffff75c87cc5f3150eefc1c04c0246e7e0b370e64b17d6226c44b333a6f4ca14b49c000000006b483045022100e0d85fece671d367c8d442a96230954cdda4b9cf95e9edc763616d05d93e944302202330d520408d909575c5f6976cc405b3042673b601f4f2140b2e4d447e671c47012103c43afccd37aae7107f5a43f5b7b223d034e7583b77c8cd1084d86895a7341abffeffffff02ebb10f00000000001976a9144ef88a0b04e3ad6d1888da4be260d6735e0d308488ac508c1e000000000017a91476c0c8f2fc403c5edaea365f6a284317b9cdf7258700000000";
    let mut transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut byte_slice = transaction_bytes.as_slice();
    transaction_bytes.clear(); // clear the vector elements while there is another reference to its elements
    // This does not work because its single writer mutable reference OR mutilple readers. Here we are trying to do both which is not allowed by Rust compiler.


    println!("byte_slice memory address before calling read_version: {:p}\n", byte_slice);

    let version = read_version(&mut byte_slice);


    println!("byte_slice memory address after calling read_version: {:p}\n", byte_slice);
    println!("byte_slice after calling read_version: {:?}", byte_slice);
    println!("Version: {}", version);

    let mut input_count = [0; 1];
    byte_slice.read(&mut input_count).unwrap();
    assert_eq!(input_count, [2_u8]);

    let mut vec = ByteVector(vec![1, 0, 4, 0, 3]);
    println!("Vec: {}", vec);

    // vec deref to slice, so slice's sort method is available to vec
    vec.0.sort(); 
    println!("Sorted Vec: {}", vec);

    let mut s1 = String::from("hello");
    let len = calculate_length(&mut s1);
    println!("The length of '{}' is {}.", &s1, len);

    let arr = [0, 1, 2, 3];
    let len = calculate_len(arr);
    println!("The len of {:?} is {}.", arr, len);

    let v = vec![1, 2, 3, 4, 5, 6];
    let r = &v;
    let aside = v; // vector data on the heap is now moved into `aside` and `v` becomes uninitialized
    // r[0] // r still points to v, which doesn't point to anything and so is a dangling pointer
}

fn calculate_length(s: &mut String) -> usize {
    s.pop();
    s.len()
}

fn calculate_len(mut arr: [u8; 4]) -> usize {
    arr.len()
}