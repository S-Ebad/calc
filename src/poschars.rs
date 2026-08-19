use std::iter::Peekable;

/// New Iterator to return the position
#[derive(Debug, Clone)]
pub struct PosChars<I>
where
    I: Iterator<Item = char>,
{
    inner: Peekable<I>,
    pos: usize,
}

impl<I> Iterator for PosChars<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.inner.next()?;
        self.pos += c.len_utf8();

        Some(c)
    }
}

impl<I> PosChars<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(inner: Peekable<I>) -> Self {
        Self { inner, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.inner.peek()
    }
}

impl<I> std::ops::DerefMut for PosChars<I>
where
    I: Iterator<Item = char>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I> std::ops::Deref for PosChars<I>
where
    I: Iterator<Item = char>,
{
    type Target = Peekable<I>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
