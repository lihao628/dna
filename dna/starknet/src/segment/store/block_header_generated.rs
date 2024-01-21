// automatically generated by the FlatBuffers compiler, do not modify
// @generated
extern crate alloc;
extern crate flatbuffers;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem;
use core::cmp::Ordering;
use self::flatbuffers::{EndianScalar, Follow};
use super::*;
pub enum BlockHeaderOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct BlockHeader<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for BlockHeader<'a> {
  type Inner = BlockHeader<'a>;
  #[inline]
  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table::new(buf, loc) }
  }
}

impl<'a> BlockHeader<'a> {
  pub const VT_BLOCK_HASH: flatbuffers::VOffsetT = 4;
  pub const VT_PARENT_HASH: flatbuffers::VOffsetT = 6;
  pub const VT_BLOCK_NUMBER: flatbuffers::VOffsetT = 8;
  pub const VT_NEW_ROOT: flatbuffers::VOffsetT = 10;
  pub const VT_TIMESTAMP: flatbuffers::VOffsetT = 12;
  pub const VT_SEQUENCER_ADDRESS: flatbuffers::VOffsetT = 14;
  pub const VT_L1_GAS_PRICE: flatbuffers::VOffsetT = 16;
  pub const VT_STARKNET_VERSION: flatbuffers::VOffsetT = 18;

  pub const fn get_fully_qualified_name() -> &'static str {
    "BlockHeader"
  }

  #[inline]
  pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    BlockHeader { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args BlockHeaderArgs<'args>
  ) -> flatbuffers::WIPOffset<BlockHeader<'bldr>> {
    let mut builder = BlockHeaderBuilder::new(_fbb);
    builder.add_timestamp(args.timestamp);
    builder.add_block_number(args.block_number);
    if let Some(x) = args.starknet_version { builder.add_starknet_version(x); }
    if let Some(x) = args.l1_gas_price { builder.add_l1_gas_price(x); }
    if let Some(x) = args.sequencer_address { builder.add_sequencer_address(x); }
    if let Some(x) = args.new_root { builder.add_new_root(x); }
    if let Some(x) = args.parent_hash { builder.add_parent_hash(x); }
    if let Some(x) = args.block_hash { builder.add_block_hash(x); }
    builder.finish()
  }


  #[inline]
  pub fn block_hash(&self) -> Option<&'a FieldElement> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<FieldElement>(BlockHeader::VT_BLOCK_HASH, None)}
  }
  #[inline]
  pub fn parent_hash(&self) -> Option<&'a FieldElement> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<FieldElement>(BlockHeader::VT_PARENT_HASH, None)}
  }
  #[inline]
  pub fn block_number(&self) -> u64 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<u64>(BlockHeader::VT_BLOCK_NUMBER, Some(0)).unwrap()}
  }
  #[inline]
  pub fn new_root(&self) -> Option<&'a FieldElement> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<FieldElement>(BlockHeader::VT_NEW_ROOT, None)}
  }
  #[inline]
  pub fn timestamp(&self) -> u64 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<u64>(BlockHeader::VT_TIMESTAMP, Some(0)).unwrap()}
  }
  #[inline]
  pub fn sequencer_address(&self) -> Option<&'a FieldElement> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<FieldElement>(BlockHeader::VT_SEQUENCER_ADDRESS, None)}
  }
  #[inline]
  pub fn l1_gas_price(&self) -> Option<ResourcePrice<'a>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<ResourcePrice>>(BlockHeader::VT_L1_GAS_PRICE, None)}
  }
  #[inline]
  pub fn starknet_version(&self) -> Option<&'a str> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(BlockHeader::VT_STARKNET_VERSION, None)}
  }
}

impl flatbuffers::Verifiable for BlockHeader<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<FieldElement>("block_hash", Self::VT_BLOCK_HASH, false)?
     .visit_field::<FieldElement>("parent_hash", Self::VT_PARENT_HASH, false)?
     .visit_field::<u64>("block_number", Self::VT_BLOCK_NUMBER, false)?
     .visit_field::<FieldElement>("new_root", Self::VT_NEW_ROOT, false)?
     .visit_field::<u64>("timestamp", Self::VT_TIMESTAMP, false)?
     .visit_field::<FieldElement>("sequencer_address", Self::VT_SEQUENCER_ADDRESS, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<ResourcePrice>>("l1_gas_price", Self::VT_L1_GAS_PRICE, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<&str>>("starknet_version", Self::VT_STARKNET_VERSION, false)?
     .finish();
    Ok(())
  }
}
pub struct BlockHeaderArgs<'a> {
    pub block_hash: Option<&'a FieldElement>,
    pub parent_hash: Option<&'a FieldElement>,
    pub block_number: u64,
    pub new_root: Option<&'a FieldElement>,
    pub timestamp: u64,
    pub sequencer_address: Option<&'a FieldElement>,
    pub l1_gas_price: Option<flatbuffers::WIPOffset<ResourcePrice<'a>>>,
    pub starknet_version: Option<flatbuffers::WIPOffset<&'a str>>,
}
impl<'a> Default for BlockHeaderArgs<'a> {
  #[inline]
  fn default() -> Self {
    BlockHeaderArgs {
      block_hash: None,
      parent_hash: None,
      block_number: 0,
      new_root: None,
      timestamp: 0,
      sequencer_address: None,
      l1_gas_price: None,
      starknet_version: None,
    }
  }
}

pub struct BlockHeaderBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> BlockHeaderBuilder<'a, 'b> {
  #[inline]
  pub fn add_block_hash(&mut self, block_hash: &FieldElement) {
    self.fbb_.push_slot_always::<&FieldElement>(BlockHeader::VT_BLOCK_HASH, block_hash);
  }
  #[inline]
  pub fn add_parent_hash(&mut self, parent_hash: &FieldElement) {
    self.fbb_.push_slot_always::<&FieldElement>(BlockHeader::VT_PARENT_HASH, parent_hash);
  }
  #[inline]
  pub fn add_block_number(&mut self, block_number: u64) {
    self.fbb_.push_slot::<u64>(BlockHeader::VT_BLOCK_NUMBER, block_number, 0);
  }
  #[inline]
  pub fn add_new_root(&mut self, new_root: &FieldElement) {
    self.fbb_.push_slot_always::<&FieldElement>(BlockHeader::VT_NEW_ROOT, new_root);
  }
  #[inline]
  pub fn add_timestamp(&mut self, timestamp: u64) {
    self.fbb_.push_slot::<u64>(BlockHeader::VT_TIMESTAMP, timestamp, 0);
  }
  #[inline]
  pub fn add_sequencer_address(&mut self, sequencer_address: &FieldElement) {
    self.fbb_.push_slot_always::<&FieldElement>(BlockHeader::VT_SEQUENCER_ADDRESS, sequencer_address);
  }
  #[inline]
  pub fn add_l1_gas_price(&mut self, l1_gas_price: flatbuffers::WIPOffset<ResourcePrice<'b >>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<ResourcePrice>>(BlockHeader::VT_L1_GAS_PRICE, l1_gas_price);
  }
  #[inline]
  pub fn add_starknet_version(&mut self, starknet_version: flatbuffers::WIPOffset<&'b  str>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(BlockHeader::VT_STARKNET_VERSION, starknet_version);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> BlockHeaderBuilder<'a, 'b> {
    let start = _fbb.start_table();
    BlockHeaderBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<BlockHeader<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for BlockHeader<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("BlockHeader");
      ds.field("block_hash", &self.block_hash());
      ds.field("parent_hash", &self.parent_hash());
      ds.field("block_number", &self.block_number());
      ds.field("new_root", &self.new_root());
      ds.field("timestamp", &self.timestamp());
      ds.field("sequencer_address", &self.sequencer_address());
      ds.field("l1_gas_price", &self.l1_gas_price());
      ds.field("starknet_version", &self.starknet_version());
      ds.finish()
  }
}
