use std::{
    fmt::Display,
    iter::{Enumerate, Peekable},
    str::{Chars, FromStr},
};

use crate::util::errors::{ParseError, ParseErrorKind};

/// ANSI 转义字符包装。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct ANSI(Vec<u8>);

impl ANSI {
    /// 解析ANSI中的数字。
    #[inline(always)]
    fn parse_num(
        iter: &mut Peekable<Enumerate<Chars<'_>>>,
        offset: usize,
    ) -> Result<u8, ParseError> {
        fn is_number(c: char) -> bool {
            c as u8 >= b'0' && c as u8 - b'0' < 10
        }
        if let Some(&(_, c)) = iter.peek()
            && is_number(c)
        {
        } else {
            return Err(ParseError::new(ParseErrorKind::NotANSI, offset));
        };
        let mut res = 0u8;
        let mut time = 0;
        while let Some(&(idx, c)) = iter.peek()
            && is_number(c)
            && time < 3
        {
            let _ = iter.next();
            res = res
                .checked_mul(10)
                .ok_or(ParseError::new(ParseErrorKind::NotANSI, idx))?;
            res = res
                .checked_add(c as u8 - b'0')
                .ok_or(ParseError::new(ParseErrorKind::NotANSI, idx))?;
            time += 1;
        }
        Ok(res)
    }
}

impl<T: IntoIterator<Item = u8>> From<T> for ANSI {
    fn from(value: T) -> Self {
        Self(value.into_iter().collect())
    }
}

impl FromStr for ANSI {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = Vec::new();
        let mut iter = s.chars().enumerate().peekable();
        let Some((_, '\x1b')) = iter.next() else {
            return Err(ParseError::new(ParseErrorKind::NotANSI, 0));
        };
        while let Some((idx, c)) = iter.next() {
            match c {
                '[' | ';' => res.push(Self::parse_num(&mut iter, idx)?),
                'm' => {
                    let None = iter.next() else {
                        return Err(ParseError::new(ParseErrorKind::NotANSI, idx));
                    };
                    break;
                }
                _ => return Err(ParseError::new(ParseErrorKind::NotANSI, idx)),
            }
        }
        Ok(Self(res))
    }
}

impl Display for ANSI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        write!(f, "\x1b[{}", self.0[0])?;
        for &num in self.0.iter().skip(1) {
            write!(f, ";{}", num)?;
        }
        write!(f, "m")?;
        Ok(())
    }
}

#[allow(clippy::bool_assert_comparison)]
#[allow(unused)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            "\x1b[1;4;5;7;38;255;128m".parse::<ANSI>().unwrap(),
            vec![1, 4, 5, 7, 38, 255, 128].into()
        );
        assert!("".parse::<ANSI>().is_err());
        assert!("\x1b[1111;1m".parse::<ANSI>().is_err());
        assert!("\x1b[1;256m".parse::<ANSI>().is_err())
    }
}
