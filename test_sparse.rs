use std::sync::Arc;

fn main() {
    // Test sparse file behavior
    let mut data: Vec<u8> = Vec::new();
    
    data.resize(32, 0);
    data[0] = b'0';
    data[1] = b'0';
    data[10] = b'1';
    data[11] = b'0';
    data[20] = b'2';
    data[21] = b'0';
    data[30] = b'3';
    data[31] = b'0';
    
    println!("Data length: {}", data.len());
    println!("Data: {:?}", data);
    
    let mut buf = [0u8; 10];
    let n = 32.min(buf.len());
    println!("Should read: {}", n);
}
