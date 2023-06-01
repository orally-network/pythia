// use std::str::FromStr;

// use anyhow::Result;

// use ic_web3::{types::H160, contract::{Error, tokens::Tokenizable, Contract}, ethabi::Token, transports::ICHttp, Web3};

// const MULTICALL3_ABI: &[u8] = include_bytes!("../../assets/Multicall3ABI.json");
// const MULTICALL3_CONTRACT_ADDRESS: &str = "0xcA11bde05977b3631167028862bE2a173976CA11";
// const MULTICALL3_AGGREGATE3_FUNCTION: &str = "aggregate3";

// pub struct Call3 {
//     pub target: H160,
//     pub allow_failure: bool,
//     pub call_data: Vec<u8>,
// }

// impl Tokenizable for Call3 {
//     fn from_token(token: Token) -> Result<Self, Error>
//     where
//         Self: Sized {
//         if let Token::Tuple(tokens) = token {
//             if tokens.len() != 3 {
//                 return Err(Error::InvalidOutputType("invalid tokens number".into()));
//             }

//             if let (
//                 Token::Address(target),
//                 Token::Bool(allow_failure),
//                 Token::Bytes(call_data)
//             ) = (tokens[0].clone(), tokens[1].clone(), tokens[2].clone()) {
//                 return Ok(Self {
//                     target,
//                     allow_failure,
//                     call_data,
//                 });
//             }
//         }

//         Err(Error::InvalidOutputType("invalid tokens".into()))
//     }

//     fn into_token(self) -> Token {
//         return Token::Tuple(vec![
//             Token::Address(self.target),
//             Token::Bool(self.allow_failure),
//             Token::Bytes(self.call_data),
//         ])
//     }
// }

// pub struct MulticallResult {
//     pub success: bool,
//     pub return_data: Vec<u8>,
// }

// impl Tokenizable for MulticallResult {
//     fn from_token(token: Token) -> Result<Self, Error>
//     where
//         Self: Sized {
//             if let Token::Tuple(tokens) = token {
//                 if tokens.len() != 2 {
//                     return Err(Error::InvalidOutputType("invalid tokens number".into()));
//                 }
    
//                 if let (
//                     Token::Bool(success),
//                     Token::Bytes(return_data)
//                 ) = (tokens[0].clone(), tokens[1].clone()) {
//                     return Ok(Self {
//                         success,
//                         return_data,
//                     });
//                 }
//             }
    
//             Err(Error::InvalidOutputType("invalid tokens".into()))
//     }

//     fn into_token(self) -> Token {
//         return Token::Tuple(vec![
//             Token::Bool(self.success),
//             Token::Bytes(self.return_data),
//         ])
//     }
// }

// pub async fn aggregate3(w3: &Web3<ICHttp>, calls: Vec<Call3>) -> Result<Vec<MulticallResult>> {
//     let contract_addr = H160::from_str(MULTICALL3_CONTRACT_ADDRESS)
//         .expect("should be valid contract address");

//     let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL3_ABI)
//         .expect("should be valid init contract data");

//     contract.sign(
//         MULTICALL3_AGGREGATE3_FUNCTION,
//         vec![calls.iter().map(|c| c.into_token()).collect()],
//         options,
//         from,
//         key_info,
//         chain_id,
//     );
//     Ok(vec![])
// }