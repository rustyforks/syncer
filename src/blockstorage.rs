use std::collections::HashMap;
use std::sync::RwLock;
use std::cmp;
extern crate blake2;
use self::blake2::Blake2b;
use self::blake2::digest::{Input, VariableOutput};
extern crate hex;

const HASHSIZE: usize = 20;
pub type BlockHash = [u8;HASHSIZE];

#[derive(Clone)]
struct Blob {
  data: Vec<u8>,
  hash: BlockHash,
}

impl Blob {
  pub fn zero(size: usize) -> Self {
    Self::new_with_data(vec![0 as u8; size])
  }

  pub fn new_with_data(data: Vec<u8>) -> Self {
    let hash = Self::hash(&data);
    Self {
      data,
      hash,
    }
  }

  pub fn read(&self, offset: usize, bytes: usize) -> Vec<u8> {
    assert!(offset < self.data.len());
    let start = offset;
    let end = cmp::min(offset+bytes, self.data.len());
    self.data[start..end].to_vec()
  }

  pub fn write(&self, offset: usize, data: &[u8]) -> Blob {
    let start = offset;
    let end = cmp::min(offset+data.len(), self.data.len());
    let bytes = end - start;
    let mut newdata = self.data.clone();
    newdata[start..end].copy_from_slice(&data[0..bytes]);
    Self::new_with_data(newdata)
  }

  fn hash(data: &[u8]) -> BlockHash {
    let mut hasher = Blake2b::new(HASHSIZE).unwrap();
    hasher.process(data);
    let mut buf = [0u8; HASHSIZE];
    hasher.variable_result(&mut buf).unwrap();
    buf
  }
}

pub struct BlobStorage {
  blobs: RwLock<HashMap<BlockHash, Blob>>,
}

impl BlobStorage {
  pub fn new() -> Self {
    BlobStorage {
      blobs: RwLock::new(HashMap::new()),
    }
  }

  pub fn read(&self, hash: &BlockHash, offset: usize, bytes: usize) -> Vec<u8> {
    let blob = self.get_blob(hash);
    blob.read(offset, bytes)
  }

  pub fn write(&self, hash: &BlockHash, offset: usize, data: &[u8]) -> BlockHash {
    let blob = self.get_blob(hash);
    let new_blob = blob.write(offset, data);
    let hash = new_blob.hash;
    self.store_blob(new_blob);
    hash
  }

  fn get_blob(&self, hash: &BlockHash) -> Blob {
    let blobs = self.blobs.read().unwrap();
    match blobs.get(hash) {
      Some(blob) => blob.clone(),
      None => Blob::zero(1), // FIXME: return error
    }
  }

  fn store_blob(&self, blob: Blob) {
    let mut blobs = self.blobs.write().unwrap();
    blobs.insert(blob.hash, blob);
  }

  pub fn zero(size: usize) -> BlockHash {
    Blob::zero(size).hash
  }

  pub fn add_block(&self, data: &[u8]) -> BlockHash {
    let blob = Blob::new_with_data(data.to_vec());
    let hash = blob.hash;
    self.store_blob(blob);
    hash
  }
}