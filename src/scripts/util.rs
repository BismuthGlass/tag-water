fn is_whitespace(b: u8) -> bool {
    b == b' ' || b == b'\r' || b == b'\n' || b == b'\t'
}

fn parse_quoted_string(bytes: &mut T) -> String
where
    T: Iterator<Item = u8> + Clone,
{
    bytes.next();
    let mut start = bytes.clone();
    let mut str_len = 0;
    while let Some(b) = bytes.next() {
        if b != b'"' {
            str_len += 1;
        } else {
            return String::from_utf8(start.take(str_len).collect()).unwrap();
        }
    }
    panic!("Unmatched `\"`");
}

fn parse_while(bytes: &mut Peekable<T>, condition: C) -> String 
where
    T: Iterator<Item = u8> + Clone,
    C: Fn(u8) -> bool
{
    let mut start = bytes.clone();
    let mut str_len = 0;
    while let Some(b) = bytes.peek() {
        if condition(*b) {
            str_len += 1;
            bytes.next();
        } else {
            break;
        }
    }
    String::from_utf8(start.take(str_len).collect()).unwrap()
}

#[cfg[test]]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}