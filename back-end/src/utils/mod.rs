use ulid::Ulid;

pub const ULID_BIN_LEN: usize = 16;

pub fn binary_to_ulid(binary: &[u8]) -> anyhow::Result<Ulid> {
    if binary.len() != ULID_BIN_LEN {
        anyhow::bail!("Invalid binary length");
    }

    let mut bytes = [0u8; ULID_BIN_LEN];
    bytes.copy_from_slice(binary);
    let val = u128::from_be_bytes(bytes);
    Ok(val.into())
}

pub fn ulid_to_binary(ulid: Ulid) -> [u8; ULID_BIN_LEN] {
    u128::from(ulid).to_be_bytes()
}
