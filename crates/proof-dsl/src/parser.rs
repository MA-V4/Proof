use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{Lexer, Token};
use rust_decimal::Decimal;

pub fn parse(source: &str) -> Result<ProductSpec, ParseError> {
    let tokens = Lexer::new(source).tokenise()?;
    Parser::new(tokens).parse_product()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                line: 0,
                message: format!("expected {:?}, got {:?}", expected, self.peek()),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Ident(s) => Ok(s),
            other => Err(ParseError::UnexpectedToken {
                line: 0,
                message: format!("expected identifier, got {:?}", other),
            }),
        }
    }

    // ───────── top level ─────────

    fn parse_product(&mut self) -> Result<ProductSpec, ParseError> {
        self.expect(&Token::Product)?;
        let name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut spec = ProductSpec {
            name,
            jurisdiction: Jurisdiction::UK,
            regulator: Regulator::FCA,
            category: ProductCategory::Deposit,
            interest: None,
            fees: None,
            protection: None,
            obligations: None,
        };

        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => return Err(ParseError::Other("unexpected EOF in product".into())),
                Token::Jurisdiction => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    spec.jurisdiction = match self.expect_ident()?.as_str() {
                        "UK" => Jurisdiction::UK,
                        "EU" => Jurisdiction::EU,
                        "US" => Jurisdiction::US,
                        s => return Err(ParseError::Other(format!("unknown jurisdiction: {}", s))),
                    };
                }
                Token::Regulator => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    spec.regulator = match self.expect_ident()?.as_str() {
                        "FCA" => Regulator::FCA,
                        "PRA" => Regulator::PRA,
                        "CFPB" => Regulator::CFPB,
                        "EBA" => Regulator::EBA,
                        s => return Err(ParseError::Other(format!("unknown regulator: {}", s))),
                    };
                }
                Token::Category => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    spec.category = match self.expect_ident()?.as_str() {
                        "deposit" => ProductCategory::Deposit,
                        "credit" => ProductCategory::Credit,
                        "mortgage" => ProductCategory::Mortgage,
                        "investment" => ProductCategory::Investment,
                        s => return Err(ParseError::Other(format!("unknown category: {}", s))),
                    };
                }
                Token::Interest => {
                    self.advance();
                    spec.interest = Some(self.parse_interest()?);
                }
                Token::Fees => {
                    self.advance();
                    spec.fees = Some(self.parse_fees()?);
                }
                Token::Protection => {
                    self.advance();
                    spec.protection = Some(self.parse_protection()?);
                }
                Token::Obligations => {
                    self.advance();
                    spec.obligations = Some(self.parse_obligations()?);
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected token in product block: {:?}", other),
                    })
                }
            }
        }
        Ok(spec)
    }

    // ───────── interest ─────────

    fn parse_interest(&mut self) -> Result<InterestBlock, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut base_rate = Rate(Decimal::ZERO);
        let mut tiers = Vec::new();
        let mut promotional = None;
        let mut accrual = Accrual {
            frequency: AccrualFrequency::Daily,
            basis: DayCountBasis::Act365,
            compound: CompoundFrequency::Annually,
            minimum_payable: None,
        };
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => return Err(ParseError::Other("unexpected EOF in interest".into())),
                Token::BaseRate => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    base_rate = self.parse_rate_lit()?;
                }
                Token::Tiers => {
                    self.advance();
                    tiers = self.parse_tiers()?;
                }
                Token::Promotional => {
                    self.advance();
                    promotional = Some(self.parse_promotional()?);
                }
                Token::Accrual => {
                    self.advance();
                    accrual = self.parse_accrual()?;
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected token in interest: {:?}", other),
                    })
                }
            }
        }
        Ok(InterestBlock {
            base_rate,
            tiers,
            promotional,
            accrual,
        })
    }

    fn parse_tiers(&mut self) -> Result<Vec<Tier>, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut tiers = Vec::new();
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => return Err(ParseError::Other("unexpected EOF in tiers".into())),
                Token::When => {
                    self.advance();
                    let condition = self.parse_condition()?;
                    self.expect(&Token::Rate)?;
                    self.expect(&Token::Colon)?;
                    let rate = self.parse_rate_expr()?;
                    tiers.push(Tier { condition, rate });
                }
                Token::Otherwise => {
                    self.advance();
                    self.expect(&Token::Rate)?;
                    self.expect(&Token::Colon)?;
                    let rate = self.parse_rate_expr()?;
                    tiers.push(Tier {
                        condition: Condition::Otherwise,
                        rate,
                    });
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected token in tiers: {:?}", other),
                    })
                }
            }
        }
        Ok(tiers)
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let field = self.expect_ident()?;
        let op = match self.advance() {
            Token::Gte => ">=",
            Token::Lte => "<=",
            Token::Gt => ">",
            Token::Lt => "<",
            other => {
                return Err(ParseError::Other(format!(
                    "expected comparison op, got {:?}",
                    other
                )))
            }
        };
        let value = match self.advance() {
            Token::Number(n) => n,
            other => {
                return Err(ParseError::Other(format!(
                    "expected number, got {:?}",
                    other
                )))
            }
        };

        match (field.as_str(), op) {
            ("balance", ">=") => Ok(Condition::BalanceGte(value)),
            ("balance", ">") => Ok(Condition::BalanceGte(value)),
            ("balance", "<=") => Ok(Condition::BalanceLt(value)),
            ("balance", "<") => Ok(Condition::BalanceLt(value)),
            ("days_since_joined", "<=") | ("days_since_joined", "<") => {
                Ok(Condition::DaysSinceJoinedLte(to_u32(value)?))
            }
            ("product_count", ">=") | ("product_count", ">") => {
                Ok(Condition::ProductCountGte(to_u32(value)?))
            }
            (f, o) => Err(ParseError::Other(format!("unknown condition: {} {}", f, o))),
        }
    }

    fn parse_rate_lit(&mut self) -> Result<Rate, ParseError> {
        match self.advance() {
            Token::Percentage(p) => Ok(Rate(p)),
            other => Err(ParseError::InvalidRate(format!("{:?}", other))),
        }
    }

    fn parse_rate_expr(&mut self) -> Result<RateExpr, ParseError> {
        match self.peek().clone() {
            Token::Percentage(p) => {
                self.advance();
                Ok(RateExpr::Literal(Rate(p)))
            }
            Token::BaseRate => {
                self.advance();
                match self.peek().clone() {
                    Token::Plus => {
                        self.advance();
                        Ok(RateExpr::BaseRatePlus(self.parse_rate_lit()?))
                    }
                    Token::Minus => {
                        self.advance();
                        Ok(RateExpr::BaseRateMinus(self.parse_rate_lit()?))
                    }
                    _ => Ok(RateExpr::BaseRate),
                }
            }
            other => Err(ParseError::InvalidRate(format!("{:?}", other))),
        }
    }

    // ───────── promotional ─────────

    fn parse_promotional(&mut self) -> Result<Promotional, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut condition = Condition::Otherwise;
        let mut rate = RateExpr::BaseRate;
        let mut expires_after_days = 0u32;
        let mut non_renewable = false;
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => {
                    return Err(ParseError::Other("unexpected EOF in promotional".into()))
                }
                Token::Condition => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    condition = self.parse_condition()?;
                }
                Token::Rate => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    rate = self.parse_rate_expr()?;
                }
                Token::ExpiresAfter => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if let Token::Number(n) = self.advance() {
                        expires_after_days = to_u32(n)?;
                        if self.peek() == &Token::Days {
                            self.advance();
                        }
                    }
                }
                Token::NonRenewable => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    non_renewable = self.advance() == Token::True;
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected token in promotional: {:?}", other),
                    })
                }
            }
        }
        Ok(Promotional {
            condition,
            rate,
            expires_after_days,
            non_renewable,
        })
    }

    // ───────── accrual ─────────

    fn parse_accrual(&mut self) -> Result<Accrual, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut frequency = AccrualFrequency::Daily;
        let mut basis = DayCountBasis::Act365;
        let mut compound = CompoundFrequency::Annually;
        let mut minimum_payable = None;
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => return Err(ParseError::Other("unexpected EOF in accrual".into())),
                Token::Frequency => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    frequency = match self.expect_ident()?.as_str() {
                        "daily" => AccrualFrequency::Daily,
                        "monthly" => AccrualFrequency::Monthly,
                        "quarterly" => AccrualFrequency::Quarterly,
                        "annually" => AccrualFrequency::Annually,
                        s => return Err(ParseError::Other(format!("unknown frequency: {}", s))),
                    };
                }
                Token::Basis => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    basis = match self.expect_ident()?.as_str() {
                        "ACT/365" => DayCountBasis::Act365,
                        "ACT/360" => DayCountBasis::Act360,
                        "30/360" => DayCountBasis::Thirty360,
                        s => return Err(ParseError::Other(format!("unknown basis: {}", s))),
                    };
                }
                Token::Compound => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    compound = match self.expect_ident()?.as_str() {
                        "daily" => CompoundFrequency::Daily,
                        "monthly" => CompoundFrequency::Monthly,
                        "quarterly" => CompoundFrequency::Quarterly,
                        "annually" => CompoundFrequency::Annually,
                        s => return Err(ParseError::Other(format!("unknown compound: {}", s))),
                    };
                }
                Token::MinimumPayable => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if let Token::Money { amount, .. } = self.advance() {
                        minimum_payable = Some(amount);
                    }
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected token in accrual: {:?}", other),
                    })
                }
            }
        }
        Ok(Accrual {
            frequency,
            basis,
            compound,
            minimum_payable,
        })
    }

    // ───────── fees ─────────

    fn parse_fees(&mut self) -> Result<FeesBlock, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut fees = Vec::new();
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Eof => return Err(ParseError::Other("unexpected EOF in fees".into())),
                Token::Fee => {
                    self.advance();
                    let name = match self.advance() {
                        Token::Str(s) => s,
                        other => {
                            return Err(ParseError::Other(format!(
                                "expected fee name, got {:?}",
                                other
                            )))
                        }
                    };
                    self.expect(&Token::LBrace)?;
                    let mut amount = FeeAmount::Fixed(Decimal::ZERO);
                    let mut waivable = false;
                    loop {
                        match self.peek().clone() {
                            Token::RBrace => {
                                self.advance();
                                break;
                            }
                            Token::Ident(ref s) if s == "amount" => {
                                self.advance();
                                self.expect(&Token::Colon)?;
                                amount = match self.advance() {
                                    Token::Money { amount: a, .. } => FeeAmount::Fixed(a),
                                    Token::Percentage(p) => FeeAmount::Percentage(Rate(p)),
                                    other => {
                                        return Err(ParseError::Other(format!(
                                            "expected fee amount, got {:?}",
                                            other
                                        )))
                                    }
                                };
                            }
                            Token::Waivable => {
                                self.advance();
                                self.expect(&Token::Colon)?;
                                waivable = self.advance() == Token::True;
                            }
                            other => {
                                return Err(ParseError::UnexpectedToken {
                                    line: 0,
                                    message: format!("{:?}", other),
                                })
                            }
                        }
                    }
                    fees.push(Fee {
                        name,
                        amount,
                        waivable,
                    });
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("unexpected in fees: {:?}", other),
                    })
                }
            }
        }
        Ok(FeesBlock { fees })
    }

    // ───────── protection ─────────

    fn parse_protection(&mut self) -> Result<ProtectionBlock, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut scheme = String::new();
        let mut limit = Decimal::ZERO;
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::Scheme => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    scheme = self.expect_ident()?;
                }
                Token::Limit => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if let Token::Money { amount, .. } = self.advance() {
                        limit = amount;
                    }
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("{:?}", other),
                    })
                }
            }
        }
        Ok(ProtectionBlock { scheme, limit })
    }

    // ───────── obligations ─────────

    fn parse_obligations(&mut self) -> Result<ObligationsBlock, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut cooling_off_days = None;
        let mut rate_change_notice_days = None;
        let mut annual_summary = false;
        loop {
            match self.peek().clone() {
                Token::RBrace => {
                    self.advance();
                    break;
                }
                Token::CoolingOff => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if let Token::Number(n) = self.advance() {
                        if self.peek() == &Token::Days {
                            self.advance();
                        }
                        cooling_off_days = Some(to_u32(n)?);
                    }
                }
                Token::RateChangeNotice => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if let Token::Number(n) = self.advance() {
                        if self.peek() == &Token::Days {
                            self.advance();
                        }
                        rate_change_notice_days = Some(to_u32(n)?);
                    }
                }
                Token::AnnualSummary => {
                    self.advance();
                    self.expect(&Token::Colon)?;
                    if self.peek() == &Token::Required {
                        self.advance();
                    }
                    annual_summary = true;
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: 0,
                        message: format!("{:?}", other),
                    })
                }
            }
        }
        Ok(ObligationsBlock {
            cooling_off_days,
            rate_change_notice_days,
            annual_summary,
        })
    }
}

fn to_u32(d: Decimal) -> Result<u32, ParseError> {
    d.to_string()
        .parse::<u32>()
        .map_err(|_| ParseError::Other(format!("expected integer, got {}", d)))
}
