/// Removes 1 character from the end of an iterator if it matches needle
pub struct RTrimIterator<I> {
    iter: I,
    needle: char,
    buffer: Option<char>,
}

impl<I> Iterator for RTrimIterator<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<I::Item> {
        loop {
            match (self.buffer, self.iter.next()) {
                (None, None) => {
                    // We reached the end of the list and have an empty buffer, meaning the string did not end with the needle character.
                    return None;
                }
                (None, Some(chr)) if chr == self.needle => {
                    // We came across an instance of needle, buffer it and loop again because we do not know if this is the end of the string.
                    self.buffer = Some(chr);
                }
                (None, Some(chr)) => {
                    // We have an empty buffer and the next character is not the needle character, return it immediately.
                    return Some(chr);
                }
                (Some(buf), None) if buf == self.needle => {
                    // We reached the end of the list and have the specified needle in the buffer where it will stay forever.
                    return None;
                }
                (Some(_), None) => {
                    // We reached the end of the list and the buffered character is not the needle character, so write it out.
                    return self.buffer.take();
                }
                (Some(_), Some(chr)) => {
                    // We have a buffered character, but it is not the end of the string, so regardless of its contents we can write it out.
                    return self.buffer.replace(chr);
                }
            };
        }
    }
}

impl<I> RTrimIterator<I> {
    pub fn new(iter: I, needle: char) -> RTrimIterator<I> {
        RTrimIterator {
            iter,
            needle,
            buffer: None,
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn no_match() {
        let input = "abcd";
        let output: String = RTrimIterator::new(input.chars(), '\n').collect();
        assert_eq!(output, input);
    }

    #[test]
    fn middle_match() {
        let input = "ab\ncd";
        let output: String = RTrimIterator::new(input.chars(), '\n').collect();
        assert_eq!(output, input);
    }

    #[test]
    fn end_match() {
        let input = "abcd\n";
        let output: String = RTrimIterator::new(input.chars(), '\n').collect();
        assert_eq!(output, "abcd");
    }

    #[test]
    fn double_match() {
        let input = "abcd\n\n";
        let output: String = RTrimIterator::new(input.chars(), '\n').collect();
        assert_eq!(output, "abcd\n");
    }
}
