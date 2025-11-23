pub trait BinarySerializable: Sized {
	fn to_bytes(
		&self,
	) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;
	fn from_bytes(
		bytes: &[u8],
	) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>;
}
