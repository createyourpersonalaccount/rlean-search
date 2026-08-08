//! XML serialization / deserialization for the rlean-search schema.

mod serde_xml;

pub use serde_xml::{index_to_xml, xml_to_index, declaration_to_xml, write_index_file, read_index_file};
