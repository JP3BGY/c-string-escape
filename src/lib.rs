use std::char;
#[derive(PartialEq,Debug)]
pub enum UnescapeError {
    UnmatchEscape,
    NotEscapeSequence(usize),
    UnivCharError(usize),
    UnicodeError(usize),
    Overflow(usize),
}
pub fn escape(input:&[u8])->String{
    let mut ret = String::new();
    for i in input.iter(){
        match i {
            b'\\'|b'\''|b'"'|b'?' => {
                ret.push('\\');
                ret.push(char::from(*i));
            },
            10u8=>{
                ret.push('\\');
                ret.push('n');
            },
            13u8=>{
                ret.push('\\');
                ret.push('r');
            },
            ch if 0x20 <= *ch && *ch < 0x7f =>{
                ret.push(char::from(*ch));
            }
            ch => {
                ret.push('\\');
                ret.push('x');
                ret=ret+&format!("{:X}",*ch);
            }
        }
    }
    ret
}
pub fn unescape(input:&str)-> Result<Vec<u8>,UnescapeError>{
    let mut itr = input.char_indices();
    let mut ret: Vec<u8> = Vec::new();
    loop{
        match itr.next() {
            None => {break;},
            Some((_pos,'\\')) =>{
                match itr.next().ok_or(UnescapeError::UnmatchEscape)?{
                    (_pos,'\'')=>{ret.push(b'\'' as u8);},
                    (_pos,'"')=>{ret.push(b'"' as u8);},
                    (_pos,'?')=>{ret.push(b'?' as u8);},
                    (_pos,'\\')=>{ret.push(b'\\' as u8);},
                    (_pos,'a')=>{ret.push(b'\x07' as u8);},
                    (_pos,'b')=>{ret.push(b'\x08' as u8);},
                    (_pos,'f')=>{ret.push(b'\x0c' as u8);},
                    (_pos,'n')=>{ret.push(b'\n' as u8);},
                    (_pos,'r')=>{ret.push(b'\r' as u8);},
                    (_pos,'t')=>{ret.push(b'\t' as u8);},
                    (_pos,'v')=>{ret.push(b'\x0b' as u8);},
                    (pos,'x')=>{
                        let mut hex=String::new();
                        let mut tmpitr=itr.clone();
                        while let Some((_pos,x)) = tmpitr.next(){
                            if ! x.is_digit(16){
                                break;
                            }
                            hex.push(x);
                            itr.next();
                        }
                        match u8::from_str_radix(&hex,16) {
                            Ok(x)=>{ret.push(x);},
                            Err(_)=>{return Err(UnescapeError::Overflow(pos-1));},
                        }
                    },
                    (pos,ch) if ch.is_digit(8) =>{
                        let mut i=0;
                        let mut tmpitr = itr.clone();
                        let mut oct = String::new();
                        oct.push(ch);
                        while let Some((_pos,x)) = tmpitr.next(){
                            if i>=2 || !x.is_digit(8){
                                break;
                            }
                            oct.push(x);
                            itr.next();
                            i=i+1;
                        }
                        match u8::from_str_radix(&oct,8){
                            Ok(x)=>{ret.push(x);},
                            Err(_)=>{return Err(UnescapeError::Overflow(pos-1));}
                        }
                    },
                    (pos,'u') =>{
                        let mut hex = String::new();
                        for _ in 0..4 {
                            let (pos,ch) = itr.next().ok_or(UnescapeError::UnivCharError(pos))?;
                            if ch.is_digit(16){
                                hex.push(ch);
                            }else{
                                return Err(UnescapeError::UnivCharError(pos));
                            }
                        };
                        let num = u32::from_str_radix(&hex,16).expect("Internal Error");
                        let s = char::from_u32(num).ok_or(UnescapeError::UnicodeError(pos-1))?;
                        let mut b = [0;3];
                        s.encode_utf8(&mut b);
                        for i in b.iter(){
                            if *i==0 {
                                break;
                            }
                            ret.push(*i);
                        }
                    },
                    (pos,'U')=>{
                        let mut hex = String::new();
                        for _ in 0..8 {
                            let (pos,ch) = itr.next().ok_or(UnescapeError::UnivCharError(pos))?;
                            if ch.is_digit(16){
                                hex.push(ch);
                            }else{
                                return Err(UnescapeError::UnivCharError(pos));
                            }
                        }
                        let num = u32::from_str_radix(&hex,16).expect("Internal Error");
                        let s = char::from_u32(num).ok_or(UnescapeError::UnicodeError(pos-1))?;
                        let mut b=[0;3];
                        s.encode_utf8(&mut b);
                        for i in b.iter(){
                            if *i==0 {
                                break;
                            }
                            ret.push(*i);
                        }
                    },
                    (pos,_)=>Err(UnescapeError::NotEscapeSequence(pos-1))?,
                };
            },
            Some((_,ch))=>{
                let mut b=[0;3];
                ch.encode_utf8(&mut b);
                for i in b.iter(){
                    if *i==0{
                        break;
                    }
                    ret.push(*i);
                };
            },
        }
    };
    Ok(ret)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unescape_test() {
        assert_eq!(unescape(r#"Hello World!"#), 
            Ok(b"Hello World!".to_vec()));
        assert_eq!(unescape(r#"\'\"\?\\\x07\x08\x0c\n\r\t\v"#), 
            Ok(b"'\"?\\\x07\x08\x0c\n\r\t\x0b".to_vec()));
        assert_eq!(unescape(r#"\x20\11\12\175"#),
            Ok(b" \t\n}".to_vec()));
        assert_eq!(unescape(r#"\u0041\U00000041"#), 
            Ok(b"AA".to_vec()));
        assert_eq!(unescape(r#"\999"#), 
            Err(UnescapeError::NotEscapeSequence(0)));
        assert_eq!(unescape(r#"ABC\777"#), 
            Err(UnescapeError::Overflow(3)));
        assert_eq!(unescape(r#"EFGH\xFFF"#), 
            Err(UnescapeError::Overflow(4)));
        assert_eq!(unescape(r#"abcde\Uffffffff"#), 
            Err(UnescapeError::UnicodeError(5)));
        assert_eq!(unescape(r#"abcde\Uffffuuuu"#), 
            Err(UnescapeError::UnivCharError(11)));
        assert_eq!(unescape(r#"\"#), 
            Err(UnescapeError::UnmatchEscape));
    }
    #[test]
    fn escape_test(){
        assert_eq!(escape(b"'\"hello\"'"), r#"\'\"hello\"\'"#);
        let mut all = [0u8;256];
        for i in 0..256{
            all[i]=i as u8;
        }
        assert_eq!(unescape(&escape(&all)), Ok(all.to_vec()));
    }
}
