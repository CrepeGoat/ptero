use std::pin::Pin;

use crate::core::Parser;


#[derive(Debug)]
struct FractalParser<P>{parser: Option<P>}

impl<'b, 'a: 'b, P: 'b> FractalParser<P>
where
    P: Parser<'b, 'a> + std::marker::Unpin
{
    fn new<F>(maker: F) -> Self
    where
        F: Fn(ParserRef<'b, 'a, P::Output>) -> P,
    {
        let mut self_ = Self {parser: None};
        let self_pin = ParserRef(Pin::new(&self_));
        let parser = maker(self_pin);
        self_.parser = Some(parser);

        self_
    }
}

impl<'b, 'a: 'b, P> Parser<'b, 'a> for FractalParser<P>
where
    P: Parser<'b, 'a>,
{
    type Output = P::Output;

    fn call(&self, s: &'a str) -> Option<Self::Output> {
        if let Some(parser) = &self.parser {
            return parser.call(s);
        }
        unreachable!();
    }
}

#[derive(Copy, Clone)]
struct ParserRef<'b, 'a: 'b, O: 'b>(Pin<&'b (dyn Parser<'b, 'a, Output = O> + std::marker::Unpin)>);

impl<'b, 'a: 'b, O: 'b> Parser<'b, 'a> for ParserRef<'b, 'a, O>
{
    type Output = O;

    fn call(&self, s: &'a str) -> Option<Self::Output> {
        self.0.call(s)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{Alt2, Seq2Rev, Digits, Str};

    #[test]
    fn test_fractal_parser() {
        let parser = FractalParser::new(|fractal|
            Alt2(
                Digits(10).post(|opt| opt.and_then(|s| s.parse::<u32>().ok())),
                Alt2(
                    Seq2Rev(
                        fractal,
                        Seq2Rev(
                            Str(&" * "),
                            fractal,
                        ),
                    ).post(|opt| opt.map(|(x1, (_s, x2))| x1*x2)),
                    Seq2Rev(
                        fractal,
                        Seq2Rev(
                            Str(&" + "),
                            fractal,
                        ),
                    ).post(|opt| opt.map(|(x1, (_s, x2))| x1+x2)),
                ),
            )
        );

        assert_eq!(parser.call(&"1 + 2 * 3 + 4"), Some(11));
    }
}
