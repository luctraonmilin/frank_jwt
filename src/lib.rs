extern crate serialize;
extern crate time;
extern crate "rust-crypto" as rust_crypto;

use serialize::base64;
use serialize::base64::{ToBase64, FromBase64};
use serialize::json;
use serialize::json::ToJson;
use serialize::json::Json;
use std::collections::TreeMap;
use rust_crypto::sha2::Sha256;
use rust_crypto::hmac::Hmac;
use rust_crypto::digest::Digest;
use rust_crypto::mac::Mac;
use std::str;

struct JwtHeader<'a> {
  alg: &'a str,
  typ: &'a str
}

pub enum Error {
  SignatureExpired,
  SignatureInvalid,
  JWTInvalid,
  IssuerInvalid,
  ExpirationInvalid,
  AudienceInvalid
}

impl<'a> ToJson for JwtHeader<'a> {
  fn to_json(&self) -> json::Json {
    let mut map = TreeMap::new();
    map.insert("typ".to_string(), self.typ.to_json());
    map.insert("alg".to_string(), self.alg.to_json());
    Json::Object(map)
  }
}

pub fn encode(payload: TreeMap<String, String>, key: &str) -> String {
  let signing_input = get_signing_input(payload);
  let signature = sign_hmac256(signing_input.as_slice(), key);
  format!("{}.{}", signing_input, signature)
}

fn get_signing_input(payload: TreeMap<String, String>) -> String {
  let header = JwtHeader{alg: "HS256", typ: "JWT"};
  let header_json_str = header.to_json();
  let encoded_header = base64_url_encode(header_json_str.to_string().as_bytes()).to_string();

  let payload = payload.into_iter().map(|(k, v)| (k, v.to_json())).collect();
  let payload_json = Json::Object(payload);
  let encoded_payload = base64_url_encode(payload_json.to_string().as_bytes()).to_string();

  format!("{}.{}", encoded_header, encoded_payload)
}

fn sign_hmac256(signing_input: &str, key: &str) -> String {
  let mut hmac = Hmac::new(Sha256::new(), key.to_string().as_bytes());
  hmac.input(signing_input.to_string().as_bytes());
  base64_url_encode(hmac.result().code())
}

fn sign_hmac384(signing_input: &str, key: &str) -> String {
  unimplemented!()
}

fn sign_hmac512(signing_input: &str, key: &str) -> String {
  unimplemented!()
}

fn base64_url_encode(bytes: &[u8]) -> String {
  bytes.to_base64(base64::URL_SAFE)
}

fn json_to_tree(input: Json) -> TreeMap<String, String> {
  match input {
    Json::Object(json_tree) => json_tree.into_iter().map(|(k, v)| (k, match v {
        Json::String(s) => s,
        _ => unreachable!()
    })).collect(),
    _ => unreachable!()
  }
}

pub fn decode(jwt: &str, key: &str, verify: bool, verify_expiration: bool) -> Result<(TreeMap<String, String>, TreeMap<String, String>), Error> {
  let (header_json, payload_json, signature, signing_input) = decoded_segments(jwt, verify);
  if verify {
    let res = verify(payload_json, signing_input.as_slice(), key, signature.as_slice());
    if !res {
      return Err(Error::SignatureInvalid)
    } 
  }

  let header = json_to_tree(header_json);
  Ok((header, payload))
}

fn decoded_segments(jwt: &str, verify: bool) -> (Json, Json, Vec<u8>, String) {
  let mut raw_segments = jwt.split_str(".");
  let header_segment = raw_segments.next().unwrap();
  let payload_segment = raw_segments.next().unwrap();
  let crypto_segment =  raw_segments.next().unwrap();
  let (header, payload) = decode_header_and_payload(header_segment, payload_segment);
  let signature = if verify {
    crypto_segment.as_bytes().from_base64().unwrap()
  } else {
    vec![]
  };

  let signing_input = format!("{}.{}", header_segment, payload_segment);
  (header, payload, signature, signing_input)
}

fn decode_header_and_payload(header_segment: &str, payload_segment: &str) -> (Json, Json) {
  fn base64_to_json(input: &str) -> Json {
    let bytes = input.as_bytes().from_base64().unwrap();
    let s = str::from_utf8(bytes.as_slice()).unwrap();
    json::from_str(s).unwrap()
  };

  let header_json = base64_to_json(header_segment);
  let payload_json = base64_to_json(payload_segment);
  (header_json, payload_json)
}

fn verify_signature(signing_input: &str, key: &str, signature_bytes: &[u8]) -> bool {
  let mut hmac = Hmac::new(Sha256::new(), key.to_string().as_bytes());
  hmac.input(signing_input.to_string().as_bytes());
  secure_compare(signature_bytes, hmac.result().code())
}

fn secure_compare(a: &[u8], b: &[u8]) -> bool {
  if a.len() != b.len() {
    return false
  }

  let mut res = 0_u8;
  for (&x, &y) in a.iter().zip(b.iter()) {
    res |= x ^ y;
  }

  res == 0
}

pub fn verify(Json, signing_input: &str, key: &str, signature_bytes: &[u8]) -> Result<TreeMap<String, String>, Error> {
  if signing_input.is_empty() || signing_input.as_slice().is_whitespace() {
    return Err(Error::JWTInvalid)
  }

  verify_signature(signing_input, key, signature_bytes);
  verify_issuer();
  verify_expiration();
  verify_audience();
}

fn verify_issuer(payload_json: Json) -> bool {
  if iss.is_empty() || signing_input.as_slice().is_whitespace() {
    return Err(Error::IssuerInvalid)
  }
}

fn verify_expiration(payload_json: Json) -> bool {
  let payload = json_to_tree(payload_json);
  if payload.contains_key("exp") {
    if exp.is_empty() || signing_input.as_slice().is_whitespace() {
     return Err(Error::ExpirationInvalid)
    }

    let exp: i64 = from_str(payload.get("exp").unwrap().as_slice()).unwrap();
    let now = time::get_time().sec;
    if exp <= now {
      return Err(Error::SignatureExpired)
    }
  }
}

fn verify_audience(payload_json: Json) -> bool {
  if aud.is_empty() || signing_input.as_slice().is_whitespace() {
    return Err(Error::AudienceInvalid)
  }
}

fn verify_subject(payload_json: Json) -> bool {
  unimplemented!()  
}

fn verify_notbefore(payload_json: Json) -> bool {
  unimplemented!()
}

fn verify_issuedat(payload_json: Json) -> bool {
  unimplemented!()
}

fn verify_jwtid(payload_json: Json) -> bool {
  unimplemented!()
}

fn verify_generic(payload_json: Json, parameter_name: String) -> bool {
  unimplemented!()
}

#[cfg(test)]
mod tests {
  extern crate time;

  use super::encode;
  use super::decode;
  use super::secure_compare;
  use std::collections::TreeMap;
  use std::time::duration::Duration;

  #[test]
  fn test_encode_and_decode_jwt() {
    let mut p1 = TreeMap::new();
    p1.insert("key1".to_string(), "val1".to_string());
    p1.insert("key2".to_string(), "val2".to_string());
    p1.insert("key3".to_string(), "val3".to_string());
    let secret = "secret123";

    let jwt = encode(p1.clone(), secret);
    let res = decode(jwt.as_slice(), secret, true, false);
    assert!(res.is_ok() && !res.is_err());
    let (_, p2) = res.ok().unwrap();
    assert_eq!(p1, p2);
  } 

  #[test]
  fn test_decode_valid_jwt() {
    let mut p1 = TreeMap::new();
    p1.insert("key11".to_string(), "val1".to_string());
    p1.insert("key22".to_string(), "val2".to_string());
    let secret = "secret123";
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJrZXkxMSI6InZhbDEiLCJrZXkyMiI6InZhbDIifQ.jrcoVcRsmQqDEzSW9qOhG1HIrzV_n3nMhykNPnGvp9c";
    let res = decode(jwt.as_slice(), secret, true, false);
    assert!(res.is_ok() && !res.is_err());
    let (_, p2) = res.ok().unwrap();
    assert_eq!(p1, p2);
  }

  #[test]
  fn test_fails_when_expired() {
    let now = time::get_time();
    let past = now + Duration::minutes(-5);
    let mut p1 = TreeMap::new();
    p1.insert("exp".to_string(), past.sec.to_string());
    p1.insert("key1".to_string(), "val1".to_string());
    let secret = "secret123";
    let jwt = encode(p1.clone(), secret);
    let res = decode(jwt.as_slice(), secret, true, true);
    assert!(!res.is_ok() && res.is_err());
  }

  #[test]
  fn test_ok_when_expired_not_verified() {
    let now = time::get_time();
    let past = now + Duration::minutes(-5);
    let mut p1 = TreeMap::new();
    p1.insert("exp".to_string(), past.sec.to_string());
    p1.insert("key1".to_string(), "val1".to_string());
    let secret = "secret123";
    let jwt = encode(p1.clone(), secret);
    let res = decode(jwt.as_slice(), secret, true, false);
    assert!(res.is_ok() && !res.is_err());
  }
  
  #[test]
  fn test_secure_compare_same_strings() {
    let str1 = "same same".as_bytes();
    let str2 = "same same".as_bytes();
    let res = secure_compare(str1, str2);
    assert!(res);
  }

  #[test]
  fn test_fails_when_secure_compare_different_strings() {
    let str1 = "same same".as_bytes();
    let str2 = "same same but different".as_bytes();
    let res = secure_compare(str1, str2);
    assert!(!res);

    let str3 = "same same".as_bytes();
    let str4 = "same ssss".as_bytes();
    let res2 = secure_compare(str3, str4);
    assert!(!res2);
  }
}