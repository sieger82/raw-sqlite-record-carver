// use std::io;
// use std::io::prelude::*;
use std::io::{BufReader, Read};
use std::fs::File;
use std::str;
// use std::time::Instant;
use std::convert::TryInto;
use std::env;


// enum SqliteField {
//     Int(i32),
//     Float(f64),
//     Text(String),
//     Blob(Vec<u8>),
// }

fn demo<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

// fn pause() {
//     let mut stdin = io::stdin();
//     let mut stdout = io::stdout();

//     // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
//     write!(stdout, "Press any key to continue...").unwrap();
//     stdout.flush().unwrap();

//     // Read a single byte and discard
//     let _ = stdin.read(&mut [0u8]).unwrap();
// }

// fn read_big_endian_varint<R: Read>(f: &mut R) -> Result<u64, &'static str> {
// fn read_big_endian_varint<R: Read>(f: &mut R) -> Result<(u64,u64), std::io::Error> {
fn read_big_endian_varint<R: Read>(mut f: R) -> Result<(u64,u64), std::io::Error> {
    let mut buffer = [0u8; 1];
    let mut result = 0u64;
    let mut bytesread = 0u64;
    for bytenum in 0..9 {
        // println!("read_varint: bytenum: {}", bytenum);
        f.read_exact(&mut buffer[..] )?;
        if bytenum < 8 && buffer[0] & 0x80 == 128 {
            result += (buffer[0] & 0x7f) as u64;
            result = result << 7;
            bytesread += 1;
            // println!("read_varint: result: {}", result);
        } else if bytenum < 8 && buffer[0] & 0x80 != 128 {
            result += (buffer[0] & 0x7f) as u64;
            // println!("read_varint: result: {}", result);
            // println!("read_varint: breaking");
            bytesread += 1;
            break; 
        } else {  // last byte
            result += buffer[0] as u64;
            // println!("read_varint: result: {}", result);
            // println!("read_varint: breaking");
            bytesread += 1;
            break; 
        } 
    }
    Ok( (result, bytesread) )
}

fn read_raw_fieldspec<R: Read>(mut f: R, header_length: u64, header_length_numbytes: u64) -> Result<Vec<(&'static str,u64,u64)>,std::io::Error>{
    let mut result_vec = Vec::new();
    let mut bytes_read = 0;
    while bytes_read < (header_length - header_length_numbytes){
        // println!("read_raw_fieldspec: reading byte num: {}", bytes_read);
        let field_value = read_big_endian_varint(&mut f)?;
        bytes_read += field_value.1;
        if field_value.0 == 0 {
            result_vec.push( ("null",0,field_value.1) ); // Value is a NULL, data size = 0
        } else if field_value.0 == 1 {
            result_vec.push( ("8bit",1,field_value.1) ); // Value is an 8-bit twos-complement integer. 
        } else if field_value.0 == 2 {
            result_vec.push( ("16bit",2,field_value.1) ); // Value is a big-endian 16-bit twos-complement integer.
        } else if field_value.0 == 3 {
            result_vec.push( ("24bit",3,field_value.1) ); // Value is a big-endian 24-bit twos-complement integer. 
        } else if field_value.0 == 4 {
            result_vec.push( ("32bit",4,field_value.1) ); // Value is a big-endian 32-bit twos-complement integer. 
        } else if field_value.0 == 5 {
            result_vec.push( ("48bit",6,field_value.1) ); // Value is a big-endian 48-bit twos-complement integer. 
        } else if field_value.0 == 6 {
            result_vec.push( ("64bit",8,field_value.1) ); // Value is a big-endian 64-bit twos-complement integer. 
        } else if field_value.0 == 7 {
            result_vec.push( ("float",8,field_value.1) ); // Value is a big-endian IEEE 754-2008 64-bit floating point number.
        } else if field_value.0 == 8 {
            result_vec.push( ("zero",0,field_value.1) ); // Value is the integer 0
        } else if field_value.0 == 9 {
            result_vec.push( ("one",0,field_value.1) ); // Value is the integer 1
        } else if field_value.0 >= 12 && field_value.0 % 2 == 0 {
            result_vec.push( ("blob",(field_value.0-12)/2,field_value.1) ); // Value is a BLOB that is (N-12)/2 bytes in length. 
        } else if field_value.0 >= 13 && field_value.0 % 2 == 1 {
            result_vec.push( ("text",(field_value.0-13)/2,field_value.1) ); // Value is a string in the text encoding and (N-13)/2 bytes in length. The nul terminator is not stored. 
        }
    }
    // println!("read_raw_fieldspec: {:?}", result_vec);
    Ok(result_vec)
}

// fn parse_record_content<R: Read>(mut f: R, field_spec_raw: Vec<(&str,u64,u64)>) -> Result<Vec<SqliteField>,std::io::Error> {
//     let mut result_vec = Vec::<SqliteField>::new();
//     for field in field_spec_raw {
//         // println!("{:?}", field);
//         let mut buffer = vec![0u8; field.1 as usize];
//         f.read_exact(&mut buffer[..])?;
//         if field.0 == "text" {
//             // result_vec.push(SqliteField::Text(str::from_utf8(&buffer).unwrap().to_string()));
//             let mystring = str::from_utf8(&buffer);
//             if mystring.is_ok() {
//                 // println!("{:?}", buffer);
//                 // println!("{}", str::from_utf8(&buffer).unwrap());
//                 print!("{}, ", str::from_utf8(&buffer).unwrap());
//             }

//         }
//         // println!("{:?}", buffer);

//     }
//     print!("\n");
//     // println!("{:?}", field_spec_raw);
//     Ok(vec![SqliteField::Int(3)])
// }

fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter()
                                 .map(|b| format!("{:02X}", b))
                                 .collect();
    strs.join(" ")
}


fn parse_record_content<R: Read>(mut f: R, field_spec_raw: Vec<(&str,u64,u64)>) -> Result<String,std::io::Error> {
    let mut result_string = String::new();
    for field in field_spec_raw {
        // println!("{:?}", field);
        let mut buffer = vec![0u8; field.1 as usize];
        f.read_exact(&mut buffer[..])?;
        if field.0 == "null" {
            result_string.push_str("'null'")
        } else if field.0 == "8bit" {
            let myint = i8::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string()); 
        } else if field.0 == "16bit" {
            let myint = i16::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string());
        } else if field.0 == "24bit" {
            // make it 4 bytes long
            buffer.insert(0,0);
            let myint = i32::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string());
        } else if field.0 == "32bit" {
            let myint = i32::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string());
        } else if field.0 == "48bit" {
            // make it 8 bytes long
            buffer.insert(0, 0);
            buffer.insert(0,0);
            let myint = i64::from_be_bytes(demo(buffer));
            // let twos_complement = (myint as i64 * -1i64) as u64;
            // let twos_complement = myint.wrapping_neg();
            result_string.push_str(&myint.to_string()); 
        } else if field.0 == "64bit" {
            let myint = i64::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string());    
        } else if field.0 == "float" {
            let myint = f64::from_be_bytes(demo(buffer));
            result_string.push_str(&myint.to_string());    
        } else if field.0 == "zero" {
            let myint = 0;
            result_string.push_str(&myint.to_string());
        } else if field.0 == "one" {
            let myint = 1;
            result_string.push_str(&myint.to_string());
        } else if field.0 == "blob" {
            result_string.push_str(&to_hex_string(buffer));              
        } else if field.0 == "text" {
            // result_vec.push(SqliteField::Text(str::from_utf8(&buffer).unwrap().to_string()));
            let mystring_result = str::from_utf8(&buffer);
            if mystring_result.is_ok() {
                result_string.push_str("'");
                result_string.push_str(mystring_result.unwrap());
                result_string.push_str("'");
            } else {
                result_string.push_str(&to_hex_string(buffer));
            }
            
        }
        result_string.push_str(", ")
        // println!("{:?}", buffer);

    }
    result_string.push_str("\n");
    // println!("{:?}", field_spec_raw);
    Ok(result_string)
}


// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>());
// }

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // let now = Instant::now();

    // let f = File::open("testdb")?;
    // let f = File::open("signal_backup.db")?;
    let f = File::open(&args[1])?;
    
    let mut reader = BufReader::new(f);
    
    loop {
        // set up bytes read counter to enable seek-back for next iteration
        let mut bytes_read_this_iter = 0u64;

        // try to read header (payload size)
        let payload_result = read_big_endian_varint(&mut reader)?;
        let payload_length = payload_result.0;
        let payload_numbytes = payload_result.1;
        
        bytes_read_this_iter += payload_numbytes;

        if payload_length > 0 {
            // println!("main: payload length: {}, payload_numbytes: {}", payload_length, payload_numbytes);

            // try to read row_id
            let (row_id, row_id_numbytes) = read_big_endian_varint(&mut reader)?;
            // let row_id = row_id_result.0;
            // let row_id_numbytes = row_id_result.1;
            bytes_read_this_iter += row_id_numbytes;

            if row_id > 0{
                // println!("main: row_id: {}, row_id_numbytes: {}", row_id, row_id_numbytes);

                // try read header length
                // TODO: should this be varint or u16le ??
                // let (header_length, header_length_numbytes) = read_big_endian_varint(&mut reader)?;
                let mut header_length_buffer = [0;2];
                // let read_header_length_result = 
                reader.read_exact(&mut header_length_buffer[..])?;
                let header_length = u16::from_le_bytes(header_length_buffer) as u64;
                let header_length_numbytes = 2;

                bytes_read_this_iter += header_length_numbytes;

                if header_length > 0 && header_length <= payload_length && header_length > header_length_numbytes {
                    // println!("main: header_length: {}, header_length_numbytes: {}", header_length, header_length_numbytes);
                    
                    let field_spec_raw = read_raw_fieldspec(&mut reader, header_length, header_length_numbytes)?;
                    let mut payload_size: u64 = header_length;
            
                    for field_spec_item in &field_spec_raw {
                        payload_size = payload_size.overflowing_add(field_spec_item.1).0;
                        bytes_read_this_iter += field_spec_item.2;
                    }
                    // println!("actual payload size: {}, payload length specified in header: {}", payload_size, payload_length);
                    if payload_size == payload_length {    // actual payload size matches what is specified in header
                        // println!("actual payload size matches what is specified in header");
                        let record_content = parse_record_content(&mut reader, field_spec_raw);
                        print!("{} {}", row_id, record_content.unwrap());
                        // println!("{}", now.elapsed().as_secs());
                    } else {
                        // println!("bytes read this iter: {}", bytes_read_this_iter);
                        reader.seek_relative((bytes_read_this_iter as i64 - 1)*-1)?;
                        // println!("{}", now.elapsed().as_secs());
                    }
                } else {
                    // println!("bytes read this iter: {}", bytes_read_this_iter);
                    reader.seek_relative((bytes_read_this_iter as i64 - 1)*-1)?;
                    // println!("{}", now.elapsed().as_secs());
                }
            } else {
                // println!("bytes read this iter: {}", bytes_read_this_iter);
                reader.seek_relative((bytes_read_this_iter as i64 - 1)*-1)?;
                // println!("{}", now.elapsed().as_secs());
            }
        } else {
            // println!("bytes read this iter: {}", bytes_read_this_iter);
            reader.seek_relative((bytes_read_this_iter as i64 - 1)*-1)?;
            // println!("{}", now.elapsed().as_secs());
        }
        // pause();
        // let mut payload_length = read_big_endian_varint(&mut reader);
        // let mut buffer = [0; 10];

        // let n = reader.read(&mut buffer[..])?;
        // println!("The bytes: {:02X?}", &buffer[..n]);

        // let x = reader.seek_relative(-3);

        // let n = reader.read(&mut buffer[..])?;
        // println!("The bytes: {:02X?}", &buffer[..n]);
    }


}