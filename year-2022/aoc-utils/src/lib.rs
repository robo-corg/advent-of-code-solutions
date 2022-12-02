// use std::io::{self, BufRead};
// use std::iter;

// use thiserror::Error;

// struct SpannedLinesIter<B>(iter::Enumerate<io::Lines<B>>);

// struct LineSpanned<T> {
//     line_no: usize,
//     item: T,
// }

// impl <T> LineSpanned<T> {
//     fn map<U, F>(self, f: F) -> LineSpanned<U>
//     where
//         F: FnOnce(T) -> U,
//     {
//         LineSpanned {
//             line_no: self.line_no,
//             item: f(self.item),
//         }
//     }
// }

// trait BufReaderAocUtilExt: BufRead {
//     fn spanned_lines(self) -> SpannedLinesIter<Self> {
//         SpannedLinesIter(self.lines().enumerate())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
