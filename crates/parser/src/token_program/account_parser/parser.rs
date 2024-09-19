use std::borrow::Cow;

use spl_token::{
    solana_program::{program_error::ProgramError, program_pack::Pack},
    state::{Account, Mint, Multisig},
};
use yellowstone_vixen_core::{
    AccountUpdate, ParseError, ParseResult, Parser, Prefilter, ProgramParser,
};

use crate::helpers::IntoProtoData;

#[derive(Debug)]
pub enum TokenProgramState {
    TokenAccount(Account),
    Mint(Mint),
    Multisig(Multisig),
}

impl TokenProgramState {
    pub fn try_unpack(data_bytes: &[u8]) -> ParseResult<Self> {
        match data_bytes.len() {
            Mint::LEN => Mint::unpack(data_bytes).map(Self::Mint).map_err(Into::into),
            Account::LEN => Account::unpack(data_bytes)
                .map(Self::TokenAccount)
                .map_err(Into::into),
            Multisig::LEN => Multisig::unpack(data_bytes)
                .map(Self::Multisig)
                .map_err(Into::into),
            _ => Err(ParseError::from("Invalid Account data length".to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TokenProgramAccParser;

impl Parser for TokenProgramAccParser {
    type Input = AccountUpdate;
    type Output = TokenProgramState;

    fn id(&self) -> Cow<str> {
        "yellowstone_vixen_parser::token_program::TokenProgramAccParser".into()
    }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([spl_token::ID])
            .build()
            .unwrap()
    }

    async fn parse(&self, acct: &AccountUpdate) -> ParseResult<Self::Output> {
        let inner = acct.account.as_ref().ok_or(ProgramError::InvalidArgument)?;

        TokenProgramState::try_unpack(&inner.data)
    }
}

impl ProgramParser for TokenProgramAccParser {
    #[inline]
    fn program_id(&self) -> yellowstone_vixen_core::Pubkey {
        spl_token::ID.to_bytes().into()
    }
}

#[cfg(feature = "proto")]
mod proto_parser {
    use yellowstone_vixen_proto::parser::{
        token_program_state_proto, MintProto, MultisigProto, TokenAccountProto,
        TokenProgramStateProto,
    };

    use super::{Account, IntoProtoData, Mint, Multisig, TokenProgramAccParser, TokenProgramState};

    impl IntoProtoData<TokenAccountProto> for Account {
        fn into_proto_data(self) -> TokenAccountProto {
            TokenAccountProto {
                mint: self.mint.to_string(),
                owner: self.owner.to_string(),
                amount: self.amount,
                delegate: self.delegate.map(|d| d.to_string()).into(),
                state: self.state as i32,
                is_native: self.is_native.into(),
                delegated_amount: self.delegated_amount,
                close_authority: self.close_authority.map(|ca| ca.to_string()).into(),
            }
        }
    }

    impl IntoProtoData<MintProto> for Mint {
        fn into_proto_data(self) -> MintProto {
            MintProto {
                mint_authority: self.mint_authority.map(|ma| ma.to_string()).into(),
                supply: self.supply,
                decimals: self.decimals.into(),
                is_initialized: self.is_initialized,
                freeze_authority: self.freeze_authority.map(|fa| fa.to_string()).into(),
            }
        }
    }

    impl IntoProtoData<MultisigProto> for Multisig {
        fn into_proto_data(self) -> MultisigProto {
            MultisigProto {
                m: self.m.into(),
                n: self.n.into(),
                is_initialized: self.is_initialized,
                signers: self.signers.into_iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl crate::proto::IntoProto for TokenProgramAccParser {
        type Proto = TokenProgramStateProto;

        fn into_proto(value: Self::Output) -> Self::Proto {
            let state_oneof = match value {
                TokenProgramState::TokenAccount(data) => Some(
                    token_program_state_proto::StateOneof::TokenAccount(data.into_proto_data()),
                ),
                TokenProgramState::Mint(data) => Some(token_program_state_proto::StateOneof::Mint(
                    data.into_proto_data(),
                )),
                TokenProgramState::Multisig(data) => Some(
                    token_program_state_proto::StateOneof::Multisig(data.into_proto_data()),
                ),
            };
            Self::Proto { state_oneof }
        }
    }
}
#[cfg(test)]
mod tests {
    use yellowstone_vixen_mock::{account_fixture, run_account_parse, FixtureData};

    use super::*;

    #[tokio::test]
    async fn test_mint_account_parsing() {
        let parser = TokenProgramAccParser;

        let account = account_fixture!("3SmPYPvZfEmroktLiJsgaNENuPEud3Z52zSfLQ1zJdkK");
        let state = run_account_parse!(parser, account);

        let TokenProgramState::Mint(mint) = state else {
            panic!("Invalid Account");
        };

        assert_eq!(mint.decimals, 10);
    }
}