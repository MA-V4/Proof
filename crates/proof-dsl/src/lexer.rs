use rust_decimal::Decimal;
use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Block keywords
    Product,
    Interest, Tiers, Promotional, Accrual,
    Fees, Protection, Obligations,
    // Field keywords
    Jurisdiction, Regulator, Category,
    BaseRate, When, Otherwise, Rate, Condition,
    ExpiresAfter, NonRenewable,
    Frequency, Basis, Compound, MinimumPayable,
    Fee, Waivable, Scheme, Limit,
    CoolingOff, RateChangeNotice, AnnualSummary,
    // Value keywords
    Days, Required, True, False,
    // Literals
    Ident(String),
    Str(String),
    Number(Decimal),
    Percentage(Decimal),
    Money { currency: String, amount: Decimal },
    // Operators
    Plus, Minus,
    Gte, Lte, Gt, Lt,
    // Punctuation
    LBrace, RBrace, Colon,
    // End
    Eof,
}

#[allow(dead_code)]
pub struct Lexer {
    input: Vec<char>,
    pos:   usize,
    pub line: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { input: input.chars().collect(), pos: 0, line: 1 }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch == Some('\n') { self.line += 1; }
        if self.pos < self.input.len() { self.pos += 1; }
        ch
    }

    fn skip_ws(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => { self.advance(); }
                Some('#') => { while let Some(c) = self.advance() { if c == '\n' { break; } } }
                _ => break,
            }
        }
    }

    fn read_string(&mut self) -> Result<String, ParseError> {
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.advance() {
                None | Some('\n') => return Err(ParseError::UnexpectedToken {
                    line: self.line,
                    message: "unterminated string".into(),
                }),
                Some('"') => break,
                Some(c)   => s.push(c),
            }
        }
        Ok(s)
    }

    fn read_number(&mut self) -> Decimal {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' { s.push(c); self.advance(); }
            else if c == '_'                  { self.advance(); } // separator
            else                              { break; }
        }
        s.parse::<Decimal>().unwrap_or(Decimal::ZERO)
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { s.push(c); self.advance(); }
            else { break; }
        }
        s
    }

    fn keyword(s: &str) -> Option<Token> {
        Some(match s {
            "product"            => Token::Product,
            "jurisdiction"       => Token::Jurisdiction,
            "regulator"          => Token::Regulator,
            "category"           => Token::Category,
            "interest"           => Token::Interest,
            "base_rate"          => Token::BaseRate,
            "tiers"              => Token::Tiers,
            "when"               => Token::When,
            "otherwise"          => Token::Otherwise,
            "rate"               => Token::Rate,
            "promotional"        => Token::Promotional,
            "condition"          => Token::Condition,
            "expires_after"      => Token::ExpiresAfter,
            "non_renewable"      => Token::NonRenewable,
            "accrual"            => Token::Accrual,
            "frequency"          => Token::Frequency,
            "basis"              => Token::Basis,
            "compound"           => Token::Compound,
            "minimum_payable"    => Token::MinimumPayable,
            "fees"               => Token::Fees,
            "fee"                => Token::Fee,
            "waivable"           => Token::Waivable,
            "protection"         => Token::Protection,
            "scheme"             => Token::Scheme,
            "limit"              => Token::Limit,
            "obligations"        => Token::Obligations,
            "cooling_off"        => Token::CoolingOff,
            "rate_change_notice" => Token::RateChangeNotice,
            "annual_summary"     => Token::AnnualSummary,
            "days"               => Token::Days,
            "required"           => Token::Required,
            "true"               => Token::True,
            "false"              => Token::False,
            _                    => return None,
        })
    }

    pub fn tokenise(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            let line = self.line;
            match self.peek() {
                None       => { out.push(Token::Eof); break; }
                Some('{')  => { self.advance(); out.push(Token::LBrace); }
                Some('}')  => { self.advance(); out.push(Token::RBrace); }
                Some(':')  => { self.advance(); out.push(Token::Colon);  }
                Some('+')  => { self.advance(); out.push(Token::Plus);   }
                Some('-')  => { self.advance(); out.push(Token::Minus);  }
                Some('>') => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); out.push(Token::Gte); }
                    else { out.push(Token::Gt); }
                }
                Some('<') => {
                    self.advance();
                    if self.peek() == Some('=') { self.advance(); out.push(Token::Lte); }
                    else { out.push(Token::Lt); }
                }
                Some('"') => {
                    let s = self.read_string()?;
                    out.push(Token::Str(s));
                }
                Some(c) if c.is_ascii_digit() => {
                    let n = self.read_number();
                    if self.peek() == Some('%') { self.advance(); out.push(Token::Percentage(n)); }
                    else                        { out.push(Token::Number(n)); }
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let ident = self.read_ident();

                    // Currency literal: GBP/EUR/USD followed by a number
                    if matches!(ident.as_str(), "GBP" | "EUR" | "USD") {
                        let saved = (self.pos, self.line);
                        self.skip_ws();
                        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                            let amount = self.read_number();
                            out.push(Token::Money { currency: ident, amount });
                            continue;
                        }
                        (self.pos, self.line) = saved;
                    }

                    // Day count basis: ACT/365, ACT/360
                    if ident == "ACT" && self.peek() == Some('/') {
                        self.advance();
                        let rest = self.read_ident();
                        out.push(Token::Ident(format!("ACT/{}", rest)));
                        continue;
                    }

                    out.push(Self::keyword(&ident).unwrap_or(Token::Ident(ident)));
                }
                Some(c) => {
                    let ch = c;
                    self.advance();
                    return Err(ParseError::UnexpectedToken {
                        line,
                        message: format!("unexpected character '{}'", ch),
                    });
                }
            }
        }
        Ok(out)
    }
}
