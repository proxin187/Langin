

pub fn output_string_asm(string: &str) -> String {
    let mut result: String = String::new();
    let mut index = 0;

    let bytes = string.as_bytes();

    while index < string.len() {
        if bytes[index] as char == '\\' {
            index += 1;
            if index >= bytes.len() {
                return result;
            } else if bytes[index] as char == 'n' {
                result = result + "\", 10, \"";
            }
        } else {
            result = result + &(bytes[index] as char).to_string();
        }
        index += 1;
    }

    return result;
}


