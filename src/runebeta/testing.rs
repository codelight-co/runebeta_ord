use crate::{Chain, IndexExtension};
use anyhow::Result;
use bitcoin::{
  block::{Header, Version},
  consensus::{encode::Error, Decodable},
  hash_types::TxMerkleNode,
  BlockHash, CompactTarget, Transaction, Txid,
};
use hex::FromHex;
use hyper::body::Buf;
use ordinals::Runestone;
use serde_json::json;
use std::str::FromStr;

#[test]
fn test_index_transaction_with_edicts() {
  let _block_hash = "00000000b622ddc1983ef0ee643801699cf5676f532592e68d1c8c0bcab0e903";
  let txid =
    Txid::from_str("2919534fee5ef7325059871e96876e4c8c16238da009d41734dd6e4d89d63af0").unwrap();
  let payload: &str = "020000000001021a6bd60dd2724f7e3cccf8696ddc90b9740afc6c62f5f19a4f3d16c39e5118890100000000ffffffffac5a9b44da02e1d578dffb57c66bdefa25b195617e7f0dced01fcf524414cf8f0000000000ffffffff05e9030000000000002251208c0b89163787ce8e5ce3f1262745024c91a6e488afb89f83a6ccaaa4db64b13500000000000000000e6a5d0b160000f0e29d011ee8070222020000000000002251208dc1576a4cf34a331d91d85d447173c97c30b4ef2ee4baab9ead6237dd4b09d8e803000000000000225120d02948e3c11f9035c2e225c325722d9701d1020c0ed7f8fe5320c17c56eaed69f4040000000000002251208dc1576a4cf34a331d91d85d447173c97c30b4ef2ee4baab9ead6237dd4b09d8014120378dea83e9695306c2e61e95eaccbb59183124d43621194fd9e47e0954fb4f91183b3409d27a56d048e4bdd15d30bd263b4ab8079d925a7ae17baac99142448301402eba2421f0958674c1b4de373da6e617dd7ea6e66d25fd4dab239f44461972f4c7b41fe98037305d461f2a5756ac45b7c5a6a878d10b516c1d757ff1f4bd360f00000000";
  let expect_runestone =
    json!({"edicts":[{"id":"2584944:30","amount":1000,"output":2}],"pointer":0});
  let transaction = parse_transaction(payload);
  assert!(transaction.is_ok());
  let transaction = transaction.unwrap();
  let artifact = Runestone::decipher(&transaction);
  let header = Header {
    version: Version::TWO,
    prev_blockhash: BlockHash::from_str(
      "00000000cd14fadcd151bca283c27a92738ad9376bdf8a8531e29226af6d7f9e",
    )
    .unwrap(),
    merkle_root: TxMerkleNode::from_str(
      "b54b4648d1f3ae159edae8bffd48d7db41890f9248e147daebffb18b51603bf9",
    )
    .unwrap(),
    time: 0,
    bits: CompactTarget::from_consensus(486604799),
    nonce: 1972576522,
  };
  let extension = IndexExtension::new(Chain::from_str("testnet").unwrap(), 0, header);
  let vec_out = extension.index_transaction_output(&txid, &transaction.output, artifact.as_ref());
  assert_eq!(vec_out.len(), 5);
  let txout = vec_out.get(1).unwrap();
  let runestone = serde_json::to_string(&txout.runestone).unwrap_or_default();
  assert_eq!(runestone, serde_json::to_string(&expect_runestone).unwrap());
}

fn parse_transaction(payload: &str) -> Result<Transaction, Error> {
  let buffer = Vec::from_hex(payload).unwrap();
  let mut buffer_reader = buffer.reader();
  Transaction::consensus_decode_from_finite_reader(&mut buffer_reader)
}
