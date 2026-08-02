//! Minimal JSON parser for tests (zero dependencies). Used to read run_body / map_body output.
#![allow(dead_code)]
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(BTreeMap<String, J>),
}

impl J {
    pub fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(m) => m.get(k),
            _ => None,
        }
    }
    pub fn arr(&self) -> &[J] {
        match self {
            J::Arr(a) => a,
            _ => panic!("not an array: {:?}", self),
        }
    }
    pub fn num(&self) -> f64 {
        match self {
            J::Num(n) => *n,
            _ => panic!("not a number: {:?}", self),
        }
    }
    pub fn str(&self) -> &str {
        match self {
            J::Str(s) => s,
            _ => panic!("not a string: {:?}", self),
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, J::Null)
    }
    /// numeric field as usize; panics with the key name when missing
    pub fn u(&self, k: &str) -> usize {
        self.get(k).unwrap_or_else(|| panic!("missing key {}", k)).num() as usize
    }
    /// [x,y] pair
    pub fn xy(&self) -> (usize, usize) {
        let a = self.arr();
        assert_eq!(a.len(), 2, "pair expected: {:?}", self);
        (a[0].num() as usize, a[1].num() as usize)
    }
}

pub fn parse(s: &str) -> Result<J, String> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    let v = val(&b, &mut i)?;
    ws(&b, &mut i);
    if i != b.len() {
        return Err(format!("trailing data at {}", i));
    }
    Ok(v)
}

fn ws(b: &[char], i: &mut usize) {
    while *i < b.len() && b[*i].is_whitespace() {
        *i += 1;
    }
}

fn val(b: &[char], i: &mut usize) -> Result<J, String> {
    ws(b, i);
    let c = *b.get(*i).ok_or("eof")?;
    match c {
        '{' => {
            *i += 1;
            let mut m = BTreeMap::new();
            ws(b, i);
            if b.get(*i) == Some(&'}') {
                *i += 1;
                return Ok(J::Obj(m));
            }
            loop {
                ws(b, i);
                let k = match val(b, i)? {
                    J::Str(s) => s,
                    _ => return Err("key not string".into()),
                };
                ws(b, i);
                if b.get(*i) != Some(&':') {
                    return Err(format!("expected : at {}", i));
                }
                *i += 1;
                m.insert(k, val(b, i)?);
                ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some('}') => {
                        *i += 1;
                        return Ok(J::Obj(m));
                    }
                    _ => return Err(format!("bad object at {}", i)),
                }
            }
        }
        '[' => {
            *i += 1;
            let mut a = vec![];
            ws(b, i);
            if b.get(*i) == Some(&']') {
                *i += 1;
                return Ok(J::Arr(a));
            }
            loop {
                a.push(val(b, i)?);
                ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some(']') => {
                        *i += 1;
                        return Ok(J::Arr(a));
                    }
                    _ => return Err(format!("bad array at {}", i)),
                }
            }
        }
        '"' => {
            *i += 1;
            let mut s = String::new();
            loop {
                let c = *b.get(*i).ok_or("eof in string")?;
                *i += 1;
                match c {
                    '"' => return Ok(J::Str(s)),
                    '\\' => {
                        let e = *b.get(*i).ok_or("eof in escape")?;
                        *i += 1;
                        match e {
                            '"' => s.push('"'),
                            '\\' => s.push('\\'),
                            '/' => s.push('/'),
                            'n' => s.push('\n'),
                            'r' => s.push('\r'),
                            't' => s.push('\t'),
                            'b' => s.push('\u{8}'),
                            'f' => s.push('\u{c}'),
                            'u' => {
                                let h: String = b.get(*i..*i + 4).ok_or("short unicode escape")?.iter().collect();
                                *i += 4;
                                let n = u32::from_str_radix(&h, 16).map_err(|_| "bad unicode escape")?;
                                s.push(char::from_u32(n).unwrap_or('?'));
                            }
                            _ => return Err(format!("bad escape {}", e)),
                        }
                    }
                    c if (c as u32) < 0x20 => return Err("raw control char in string".into()),
                    c => s.push(c),
                }
